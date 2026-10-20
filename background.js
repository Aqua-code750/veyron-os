// Veyron OS - Chrome Extension Background Service Worker (Manifest V3)

chrome.runtime.onInstalled.addListener(() => {
  console.log("Veyron OS Chrome Extension installed successfully.");
});

// Launch Veyron OS when the extension icon in the toolbar is clicked
chrome.action.onClicked.addListener(() => {
  chrome.tabs.create({ url: chrome.runtime.getURL("index.html") });
});
