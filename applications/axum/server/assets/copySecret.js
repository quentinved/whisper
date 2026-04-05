// Handle secret copying functionality
document.addEventListener("DOMContentLoaded", function () {
  const copyButton = document.getElementById("copy-secret");
  const secretInput = document.getElementById("secret-input");
  const popup = document.getElementById("popup-message");

  if (copyButton && secretInput) {
    copyButton.addEventListener("click", () => {
      // Select the secret
      secretInput.select();

      // Copy to clipboard
      navigator.clipboard
        .writeText(secretInput.value)
        .then(() => {
          // Show success popup
          if (popup) {
            popup.textContent = "Secret copied to clipboard!";
            popup.classList.add("show");

            // Automatically hide after delay
            setTimeout(() => {
              popup.classList.add("fade-out");
            }, 3000);

            setTimeout(() => {
              popup.classList.remove("show", "fade-out");
            }, 3500);
          }

          // Visual feedback on the button
          copyButton.classList.add("success");
          copyButton.textContent = "Copied!";

          setTimeout(() => {
            copyButton.classList.remove("success");
            copyButton.textContent = "Copy Secret";
          }, 2000);
        })
        .catch((err) => {
          console.error("Could not copy secret: ", err);
          if (popup) {
            popup.textContent = "Failed to copy secret";
            popup.classList.add("show", "error");

            setTimeout(() => {
              popup.classList.add("fade-out");
            }, 3000);

            setTimeout(() => {
              popup.classList.remove("show", "fade-out", "error");
            }, 3500);
          }
        });
    });

    // Auto focus on secret input when page loads
    setTimeout(() => {
      secretInput.focus();
      secretInput.select();
    }, 500);
  }

  // Warn user before leaving page
  window.addEventListener("beforeunload", function (e) {
    // The message won't be shown in modern browsers for security reasons,
    // but the prompt will still appear
    const message =
      "This secret will be permanently destroyed if you leave. Are you sure?";
    e.returnValue = message;
    return message;
  });

  // Automatic copy to clipboard on page load (optional feature)
  // Uncomment if you want this behavior
  /*
  setTimeout(() => {
    if (secretInput && copyButton) {
      secretInput.select();
      navigator.clipboard.writeText(secretInput.value)
        .then(() => {
          if (popup) {
            popup.classList.add("show");
            setTimeout(() => {
              popup.classList.add("fade-out");
            }, 3000);
            setTimeout(() => {
              popup.classList.remove("show", "fade-out");
            }, 3500);
          }
        })
        .catch(err => console.error("Auto-copy failed: ", err));
      }
  }, 1000);
  */
});
