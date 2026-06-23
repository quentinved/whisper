import {
  type ChatInputCommandInteraction,
  Client,
  Events,
  GatewayIntentBits,
} from "discord.js";

const token = process.env.DISCORD_TOKEN;
const whisperUrl = process.env.WHISPER_URL;

if (!token) {
  console.error("Missing DISCORD_TOKEN in environment");
  process.exit(1);
}

if (!whisperUrl) {
  console.error("Missing WHISPER_URL in environment");
  process.exit(1);
}

const DEFAULT_DURATION_SECONDS = 3600; // 1 hour
const MAX_DURATION_SECONDS = 7 * 86400; // 7 days

function parseDuration(input: string): number | null {
  const trimmed = input.trim();
  if (trimmed.length < 2) return null;

  const suffix = trimmed.at(-1);
  const numStr = trimmed.slice(0, -1);
  const value = parseInt(numStr, 10);

  if (Number.isNaN(value) || value <= 0) return null;

  let seconds: number;
  switch (suffix) {
    case "m":
      seconds = value * 60;
      break;
    case "h":
      seconds = value * 3600;
      break;
    case "d":
      seconds = value * 86400;
      break;
    default:
      return null;
  }

  if (seconds > MAX_DURATION_SECONDS) return null;

  return seconds;
}

function formatDuration(seconds: number): string {
  if (seconds >= 86400 && seconds % 86400 === 0) {
    return `${seconds / 86400} day(s)`;
  } else if (seconds >= 3600 && seconds % 3600 === 0) {
    return `${seconds / 3600} hour(s)`;
  }
  return `${seconds / 60} minute(s)`;
}

function b64urlEncode(bytes: Uint8Array): string {
  return Buffer.from(bytes).toString("base64url");
}

async function encryptSecret(
  plaintext: string,
): Promise<{ keyB64: string; payloadB64: string }> {
  const key = await crypto.subtle.generateKey(
    { name: "AES-GCM", length: 256 },
    true,
    ["encrypt"],
  );
  const nonce = crypto.getRandomValues(new Uint8Array(12));
  const ciphertext = new Uint8Array(
    await crypto.subtle.encrypt(
      { name: "AES-GCM", iv: nonce },
      key,
      new TextEncoder().encode(plaintext),
    ),
  );
  const rawKey = new Uint8Array(await crypto.subtle.exportKey("raw", key));
  const payload = new Uint8Array(12 + ciphertext.length);
  payload.set(nonce);
  payload.set(ciphertext, 12);
  return { keyB64: b64urlEncode(rawKey), payloadB64: b64urlEncode(payload) };
}

// Encrypts locally (the key never reaches the server) and stores only ciphertext.
async function createSecret(
  secret: string,
  expirationTimestamp: number,
  selfDestruct: boolean,
): Promise<{ id: string; keyB64: string }> {
  const { keyB64, payloadB64 } = await encryptSecret(secret);

  const response = await fetch(`${whisperUrl}/v1/ephemeral?source=discord`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      payload: payloadB64,
      expiration: expirationTimestamp,
      self_destruct: selfDestruct,
    }),
  });

  if (response.status !== 201) {
    throw new Error(
      `Unexpected response from Whisper server: ${response.status}`,
    );
  }

  const body = (await response.json()) as { id: string };
  return { id: body.id, keyB64 };
}

async function handleWhisper(interaction: ChatInputCommandInteraction) {
  const secret = interaction.options.getString("secret", true);
  const durationInput = interaction.options.getString("duration");
  const selfDestruct = interaction.options.getBoolean("self_destruct") ?? true;

  let durationSeconds = DEFAULT_DURATION_SECONDS;
  if (durationInput) {
    const parsed = parseDuration(durationInput);
    if (!parsed) {
      await interaction.reply({
        content:
          "Invalid duration. Use `30m`, `1h`, `24h`, or `7d` (max 7 days).",
        flags: 64, // Ephemeral
      });
      return;
    }
    durationSeconds = parsed;
  }

  await interaction.deferReply({ flags: 64 }); // Ephemeral

  try {
    const expirationTimestamp = Math.floor(Date.now() / 1000) + durationSeconds;
    const { id, keyB64 } = await createSecret(
      secret,
      expirationTimestamp,
      selfDestruct,
    );

    const shareUrl = `${whisperUrl}/get_secret?shared_secret_id=${id}#k=${keyB64}`;
    const durationDisplay = formatDuration(durationSeconds);
    const destructNote = selfDestruct
      ? "Self-destructs after first view."
      : "Can be viewed multiple times until expiration.";

    await interaction.editReply({
      content: `Secret created! Share this link:\n<${shareUrl}>\n\nExpires in ${durationDisplay}. ${destructNote}`,
    });
  } catch (error) {
    console.error("Failed to create secret:", error);
    await interaction.editReply({
      content: "Failed to create secret. Please try again.",
    });
  }
}

const client = new Client({ intents: [GatewayIntentBits.Guilds] });

client.once(Events.ClientReady, (readyClient) => {
  console.log(`Ready! Logged in as ${readyClient.user.tag}`);
});

client.on(Events.InteractionCreate, async (interaction) => {
  if (!interaction.isChatInputCommand()) return;

  if (interaction.commandName === "whisper") {
    await handleWhisper(interaction);
  }
});

client.login(token);
