const exts = [
  "7z", "a", "aar", "apk", "ar", "bz2", "br", "cab", "cpio", "deb",
  "dmg", "egg", "gz", "iso", "jar", "lha", "lz", "lz4", "lzma", "lzo",
  "mar", "pea", "rar", "rpm", "s7z", "shar", "tar", "tbz2", "tgz",
  "tlz", "txz", "war", "whl", "xpi", "zip", "zipx", "zst", "xz",
  "pak", "aac", "aiff", "ape", "au", "flac", "gsm", "it", "m3u",
  "m4a", "mid", "mod", "mp3", "mpa", "ogg", "pls", "ra", "s3m",
  "sid", "wav", "wma", "xm", "mobi", "epub", "azw1", "azw3",
  "azw4", "azw6", "azw", "cbr", "cbz", "exe", "msi", "bin",
  "command", "sh", "bat", "crx", "bash", "csh", "fish", "ksh",
  "zsh", "eot", "otf", "ttf", "woff", "woff2", "3dm", "3ds",
  "max", "avif", "bmp", "dds", "gif", "heic", "heif", "jpg",
  "jpeg", "jxl", "png", "psd", "xcf", "tga", "thm", "tif",
  "tiff", "yuv%", "ai", "eps", "ps", "svg", "dwg", "dxf",
  "gpx", "kml", "kmz", "webp", "ods", "xls", "xlsx", "csv",
  "tsv", "ics", "vcf", "ppt", "pptx", "odp", "doc", "docx",
  "ebook", "log", "md", "msg", "odt", "org", "pages", "pdf",
  "rtf", "rst", "tex", "txt", "wpd", "wps", "3g2", "3gp",
  "aaf", "asf", "avchd", "avi", "car", "dav", "drc", "flv",
  "m2v", "m2ts", "m4p", "m4v", "mkv", "mng", "mov", "mp2",
  "mp4", "mpe", "mpeg", "mpg", "mpv", "mts", "mxf", "nsv",
  "ogv", "ogm", "ogx", "qt", "rm", "rmvb", "roq", "srt",
  "svi", "vob", "webm", "wmv", "xba", "yuv"
];


var DEBUG = 0;
var LOG = function () {
  if (DEBUG) {
    var args = ['download-blocker:'].concat(arguments);
    console.log.apply(console, args);
  }
};

/*
 * List of MIME types allowed to download.
 */
var ALLOWED_CONTENT_TYPES = {};

function isRedirect(details) {
  return Math.floor(details.statusCode / 100) == 3;
};

/*
 * Extract Content-Type from <webRequest.HTTPHeaders>.
 */
function getContentType(headers) {
  for (var i = 0; i < headers.length; i++) {
    var header = headers[i];
    var name = header.name.toLowerCase();

    if (name == 'content-type') {
      // Trim suffixes after a semicolon. (e.g. this converts
      // 'plain/text; charset=utf-8' into 'plain/text')
      return header.value.split(';')[0].trim();
    }
  }
  return '';
};

/*
 * The callback function for <webRequest.onHeadersReceived>
 *
 * See the MDN manual on WebExtensions/webRequest.
 */
async function onHeadersReceived(details) {
  LOG('onHeadersReceived ', JSON.stringify(details));
  let res = {};
  let headers = details.responseHeaders;

  headers = headers.filter((header) => {
    return header.name.toLowerCase() != 'content-disposition';
  });

  const redirect = isRedirect(details);
  const url = details.url; // Get the URL here
  let check = false;

  // Check the file extension in the URL
  for (let i of exts) {
    if (url.endsWith('.' + i)) {
      check = true;
      break;
    }
  }

  // Await isLocalHostOnline to get accurate status
  const localHostOnline = await isLocalHostOnline();

  // If not a redirect and the URL has a disallowed extension
  if (!redirect && check && localHostOnline) {
    try {
      await browser.storage.local.set({ lastDownloadLink: url });
      console.log("Download link saved to storage.");
      sendJsonRequest(url);
      res.redirectUrl = 'data:javascript,';
    } catch (error) {
      console.error("Error saving download link:", error);
    }
  }

  res.responseHeaders = headers;

  LOG('A request with URL', url);
  LOG('Is redirection?', redirect);
  LOG('Return BlockingResponse', JSON.stringify(res));

  return res;
}



browser.webRequest.onHeadersReceived.addListener(
  onHeadersReceived,
  { urls: ['<all_urls>'], types: ['main_frame'] },
  ['blocking', 'responseHeaders']
);

LOG('background script initialized')

function sendJsonRequest(url) {
  const data = { value: url };

  fetch('http://127.0.0.1:3000', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(data)
  }).catch((error) => {
    console.error('Error:', error);
  });
}

async function isLocalHostOnline() {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), 3000); 

  try {
    const response = await fetch('http://127.0.0.1:3000', {
      method: 'HEAD',
      mode: 'no-cors',
      signal: controller.signal
    });
    clearTimeout(timeoutId);
    return response.ok || response.type === 'opaque';
  } catch (error) {
    if (error.name === 'AbortError') {
      console.error("Localhost check timed out");
    } else {
      console.error("Localhost check failed:", error);
    }
    return false;
  }
}

