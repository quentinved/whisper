import { REST, Routes, SlashCommandBuilder } from "discord.js";

const token = process.env.DISCORD_TOKEN;
const clientId = process.env.DISCORD_CLIENT_ID;

if (!token || !clientId) {
  console.error("Missing DISCORD_TOKEN or DISCORD_CLIENT_ID in environment");
  process.exit(1);
}

const whisperCommand = new SlashCommandBuilder()
  .setName("whisper")
  .setDescription("Create a temporary, encrypted secret share link")
  .addStringOption((option) =>
    option
      .setName("secret")
      .setDescription("The secret to share")
      .setRequired(true),
  )
  .addStringOption((option) =>
    option
      .setName("duration")
      .setDescription(
        "How long the secret lives (e.g. 30m, 1h, 24h, 7d). Default: 1h",
      )
      .setRequired(false),
  )
  .addBooleanOption((option) =>
    option
      .setName("self_destruct")
      .setDescription("Delete after first view? Default: true")
      .setRequired(false),
  );

const rest = new REST().setToken(token);

try {
  console.log("Registering /whisper slash command...");
  await rest.put(Routes.applicationCommands(clientId), {
    body: [whisperCommand.toJSON()],
  });
  console.log("Successfully registered /whisper command.");
} catch (error) {
  console.error("Failed to register command:", error);
  process.exit(1);
}
