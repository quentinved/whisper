// Toggle custom expiration field based on dropdown selection
const expirationOption = document.getElementById("expiration-input");
const customExpirationDiv = document.getElementById("custom-expiration");
const form = document.getElementById("form");

// Show or hide custom expiration field based on selected option
if (expirationOption) {
  expirationOption.addEventListener("change", () => {
    if (expirationOption.value === "custom") {
      customExpirationDiv.style.display = "block";
      customExpirationDiv.classList.add("animated");
    } else {
      customExpirationDiv.style.display = "none";
    }
  });
}

// Handle form submission to set expiration timestamp
if (form) {
  form.addEventListener("submit", (event) => {
    // Prevent default form submission - HTMX will handle it
    event.preventDefault();

    const selectedOption = expirationOption.value;
    const now = new Date();
    let timestamp;

    try {
      switch (selectedOption) {
        case "6-hours":
          timestamp = Math.floor(now.getTime() / 1000 + 6 * 60 * 60);
          break;
        case "1-day":
          timestamp = Math.floor(now.getTime() / 1000 + 24 * 60 * 60);
          break;
        case "2-days":
          timestamp = Math.floor(now.getTime() / 1000 + 2 * 24 * 60 * 60);
          break;
        case "custom":
          const customDate = document.getElementById(
            "custom-expiration-date"
          ).value;
          if (!customDate) {
            showPopup("Please select a valid custom expiration date and time.");
            return;
          }
          const customDateTime = new Date(customDate);
          if (customDateTime <= now) {
            showPopup("Expiration date must be in the future.");
            return;
          }
          timestamp = Math.floor(customDateTime.getTime() / 1000);
          break;
        default:
          showPopup("Please select a valid expiration option.");
          return;
      }

      document.getElementById("expiration").value = timestamp;

      // Show loading spinner during form submission
      const loadingSpinner = document.querySelector(".form-loading");
      const submitButtonText = document.querySelector("#secret-submit span");

      if (loadingSpinner && submitButtonText) {
        loadingSpinner.style.display = "flex";
        submitButtonText.style.opacity = "0";
      }

      // Let HTMX handle the form submission
      form.submit();
    } catch (error) {
      console.error("Error processing form:", error);
      showPopup("An error occurred. Please try again.");
    }
  });

  // Handle HTMX events
  document.body.addEventListener("htmx:afterRequest", function () {
    const loadingSpinner = document.querySelector(".form-loading");
    const submitButtonText = document.querySelector("#secret-submit span");

    if (loadingSpinner && submitButtonText) {
      loadingSpinner.style.display = "none";
      submitButtonText.style.opacity = "1";
    }
  });
}

// Handle popup notifications
function showPopup(message) {
  const popup = document.getElementById("popup-message");
  if (popup) {
    popup.textContent = message;
    popup.classList.add("show");

    setTimeout(() => {
      popup.classList.add("fade-out");
    }, 3000);

    setTimeout(() => {
      popup.classList.remove("show", "fade-out");
    }, 3500);
  }
}

// Automatically select all text in an input when clicked
document.addEventListener("click", (event) => {
  // Handle shared link input selection
  if (event.target.id === "shared-link-input") {
    event.target.select();
  }

  // Handle copy link button
  if (event.target.id === "copy-link" || event.target.closest("#copy-link")) {
    const linkInput = document.getElementById("shared-link-input");
    if (linkInput && linkInput.value) {
      navigator.clipboard
        .writeText(linkInput.value)
        .then(() => showPopup("Your link has been copied to clipboard!"))
        .catch((err) => console.error("Could not copy link:", err));
    }
  }
});

// Set minimum date for custom expiration to current date+time
document.addEventListener("DOMContentLoaded", function () {
  const customExpirationDate = document.getElementById(
    "custom-expiration-date"
  );
  if (customExpirationDate) {
    const now = new Date();
    now.setMinutes(now.getMinutes() + 5); // Add 5 minutes buffer

    // Format date for datetime-local input
    const year = now.getFullYear();
    const month = (now.getMonth() + 1).toString().padStart(2, "0");
    const day = now.getDate().toString().padStart(2, "0");
    const hours = now.getHours().toString().padStart(2, "0");
    const minutes = now.getMinutes().toString().padStart(2, "0");

    const minDateTime = `${year}-${month}-${day}T${hours}:${minutes}`;
    customExpirationDate.setAttribute("min", minDateTime);
    customExpirationDate.value = minDateTime;
  }
});
