document.addEventListener("DOMContentLoaded", () => {
  browser.storage.local.get("lastDownloadLink", (data) => {
    const downloadLink = data.lastDownloadLink || "No link intercepted yet.";
    const linkElement = document.getElementById("downloadLink");
    linkElement.innerText = downloadLink;
  });
});
