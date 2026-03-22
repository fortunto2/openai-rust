#!/usr/bin/env node

import { spawnSync } from 'node:child_process'
import { setTimeout } from 'node:timers/promises'

const args = process.argv.slice(2)

if (args.length === 0) {
  console.error(
    'Usage: node scripts/npm-publish-with-skip.mjs <command> [args...]',
  )
  process.exit(1)
}

const defaultMaxRetries = 3
const defaultRetryDelayMs = 25_000
const maxRetries = parsePositiveInt(
  process.env.NPM_PUBLISH_MAX_RETRIES,
  defaultMaxRetries,
)
const retryDelayMs = parsePositiveInt(
  process.env.NPM_PUBLISH_RETRY_DELAY_MS,
  defaultRetryDelayMs,
)

const main = async () => {
  for (let attempt = 1; attempt <= maxRetries; attempt += 1) {
    const result = spawnSync(args[0], args.slice(1), {
      stdio: 'pipe',
      encoding: 'utf8',
    })

    if (result.stdout) {
      process.stdout.write(result.stdout)
    }

    if (result.stderr) {
      process.stderr.write(result.stderr)
    }

    if (result.status === 0) {
      process.exit(0)
    }

    const combinedOutput = `${result.stdout ?? ''}\n${result.stderr ?? ''}`

    if (isAlreadyPublishedError(combinedOutput)) {
      console.warn(
        'WARNING npm publish target is already published; skipping duplicate publish',
      )
      process.exit(0)
    }

    if (
      isRateLimitError(combinedOutput) &&
      attempt < maxRetries
    ) {
      console.warn(
        `Too many requests (attempt ${attempt}/${maxRetries}); retrying after ${retryDelayMs /
          1000}s`,
      )
      await setTimeout(retryDelayMs)
      continue
    }

    if (isRateLimitError(combinedOutput)) {
      console.error(
        'Rate limit persisted after retries; refusing to continue to avoid publishing duplicates.',
      )
    }

    if (result.error) {
      console.error(result.error.message)
    }

    process.exit(result.status ?? 1)
  }
}

main().catch((err) => {
  console.error(err)
  process.exit(1)
})

function isAlreadyPublishedError(output) {
  return (
    /previously published versions/i.test(output) ||
    /cannot publish over (?:the )?previously published versions/i.test(output) ||
    /cannot publish over existing version/i.test(output)
  )
}

function isRateLimitError(output) {
  return (
    /(Too Many Requests|rate limited|rate limit exceeded)/i.test(output) ||
    /status code 429/i.test(output)
  )
}

function parsePositiveInt(value, fallback) {
  if (value == null || value === '') {
    return fallback
  }

  const parsed = Number.parseInt(value, 10)
  if (Number.isFinite(parsed) && parsed > 0) {
    return parsed
  }

  return fallback
}
