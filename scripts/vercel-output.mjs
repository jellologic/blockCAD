/**
 * Creates Vercel Build Output API structure from TanStack Start's Vite build.
 * Uses a Node.js serverless function for SSR.
 */

import { cpSync, mkdirSync, writeFileSync, readdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const webDist = join(root, "apps", "web", "dist");
const output = join(root, ".vercel", "output");

// Clean and create output structure
mkdirSync(join(output, "static"), { recursive: true });
mkdirSync(join(output, "functions", "ssr.func"), { recursive: true });

// 1. Copy static client assets
cpSync(join(webDist, "client"), join(output, "static"), { recursive: true });
console.log("  ✓ Copied static assets");

// 2. Copy server bundle into function directory
cpSync(join(webDist, "server"), join(output, "functions", "ssr.func", "server"), {
  recursive: true,
});
console.log("  ✓ Copied SSR server bundle");

// 3. Write function entry point
writeFileSync(
  join(output, "functions", "ssr.func", "index.mjs"),
  `
import server from "./server/server.js";

export default async function handler(req, res) {
  try {
    const protocol = req.headers["x-forwarded-proto"] || "https";
    const host = req.headers["x-forwarded-host"] || req.headers.host || "localhost";
    const url = new URL(req.url || "/", protocol + "://" + host);

    const headers = new Headers();
    for (const [key, value] of Object.entries(req.headers)) {
      if (value) headers.set(key, Array.isArray(value) ? value.join(", ") : value);
    }

    const webReq = new Request(url.toString(), {
      method: req.method,
      headers,
      body: req.method !== "GET" && req.method !== "HEAD" ? req : undefined,
      duplex: "half",
    });

    const response = await server.fetch(webReq);

    res.statusCode = response.status;
    response.headers.forEach((value, key) => {
      res.setHeader(key, value);
    });

    if (response.body) {
      const reader = response.body.getReader();
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        res.write(value);
      }
      res.end();
    } else {
      res.end(await response.text());
    }
  } catch (err) {
    console.error("SSR error:", err);
    res.statusCode = 500;
    res.end("Internal Server Error: " + (err.message || String(err)));
  }
}
`.trim()
);

// 4. Write function config — Node.js runtime with all server dependencies bundled
writeFileSync(
  join(output, "functions", "ssr.func", ".vc-config.json"),
  JSON.stringify(
    {
      runtime: "nodejs22.x",
      handler: "index.mjs",
      launcherType: "Nodejs",
      supportsResponseStreaming: true,
      maxDuration: 30,
    },
    null,
    2
  )
);

// 5. Copy node_modules that the server needs (the server bundle uses bare imports)
// The Vite SSR build externalizes node_modules, so we need them available
const serverNodeModules = join(root, "node_modules");
cpSync(serverNodeModules, join(output, "functions", "ssr.func", "node_modules"), {
  recursive: true,
  dereference: true,
});
console.log("  ✓ Copied node_modules for SSR function");

// 6. Write package.json for the function
writeFileSync(
  join(output, "functions", "ssr.func", "package.json"),
  JSON.stringify({ type: "module" }, null, 2)
);

// 7. Write Vercel output config with routes
writeFileSync(
  join(output, "config.json"),
  JSON.stringify(
    {
      version: 3,
      routes: [
        {
          src: "/assets/(.*)",
          headers: { "Cache-Control": "public, max-age=31536000, immutable" },
        },
        { handle: "filesystem" },
        { src: "/(.*)", dest: "/ssr" },
      ],
    },
    null,
    2
  )
);

console.log("✓ Vercel output created at .vercel/output/");
