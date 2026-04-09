# Vulnerability Report: Missing CORS Policy and Authentication in Local Streaming Server

**Title:** [Critical] Missing CORS Policy and Authentication in Local Streaming Server Allows Remote Data Exfiltration and Torrent Injection
**Target:** Stremio Desktop App (Windows)
**Severity:** Critical (CVSS 9.0+ estimated)

## 1. Summary
The Stremio desktop application runs a local HTTP server on `http://127.0.0.1:11470` to handle streaming and application settings. However, this local server lacks any CORS (Cross-Origin Resource Sharing) restrictions, Origin validation, or authentication mechanisms (such as API keys or session tokens). 

This critical misconfiguration allows any arbitrary, attacker-controlled website visited by the user to silently communicate with the Stremio local server. An attacker can extract sensitive system data, remotely modify Stremio settings, and force-download torrents without any user interaction or consent.

## 2. Vulnerability Details (Root Cause)
The `stremio-core` / `stremio-server` responds to all HTTP requests regardless of the `Origin` header. Because the server binds to `localhost` and modern browsers allow web pages to make requests to `localhost` (if CORS allows it or is missing in older implementations/specific endpoints), a malicious site can execute state-changing POST requests and read sensitive GET responses.

**Exposed Attack Surface:**
* `GET /settings`: Leaks OS username, file paths, and app version.
* `POST /settings`: Modifies server configuration (e.g., setting `cacheSize` to `null` to cause disk exhaustion).
* `GET /network-info`: Leaks local network interfaces and internal IPs (bypassing VPN anonymity).
* `POST /{infoHash}/create`: Forces the application to start downloading a torrent payload.

## 3. Proof of Concept (PoC)
To reproduce this vulnerability:
1. Ensure the Stremio desktop app is running on the victim's machine.
2. The victim visits an attacker-controlled website hosting the following JavaScript payload:

```javascript
// PoC 1: Data Leakage (Extracting Windows Username and Paths)
fetch('[http://127.0.0.1:11470/settings](http://127.0.0.1:11470/settings)')
  .then(response => response.json())
  .then(data => {
      console.log("Exposed Data: ", data);
      // Attacker can send this data to their own server
  });

// PoC 2: Silent Torrent Injection
const infoHash = "INSERT_MALICIOUS_INFOHASH_HERE";
fetch(`http://127.0.0.1:11470/${infoHash}/create`, {
    method: 'POST',
    body: JSON.stringify({ /* required payload */ })
});
```
<img src="https://camo.githubusercontent.com/6dd3cc3c378b6a8acf6728d7c3925bfa2cf2f9b8f927fd93b64b5280d77404b0/68747470733a2f2f6c2e746f7034746f702e696f2f705f333639397266766e37312e676966">

4. Impact
Complete Loss of Privacy: Attackers can fingerprint users, leak their exact Windows username, internal IP addresses, and device info, rendering VPNs or Incognito modes useless.
Resource Exhaustion / Denial of Service: Attackers can modify cacheSize to infinite, filling up the victim's hard drive, or manipulate btMaxConnections to crash the network.
Arbitrary File Download: Attackers can force the victim's machine to download arbitrary (and potentially illegal) torrents silently, turning the victim's device into a node for malicious swarms.

5. Recommended Remediation
To properly secure the local streaming server, the following defenses must be implemented:
Strict CORS Policy: The local server must explicitly reject requests where the Origin header does not match a trusted Stremio domain (e.g., app.strem.io or localhost explicitly).
