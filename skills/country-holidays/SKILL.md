---
name: country-holidays
description: Fetch country metadata and public holidays to inform agentic scheduling, planning, and cultural context.
runx:
  category: context
---

# Country Holidays

Fetch country information and public holidays for a given year.

This skill provides cultural and administrative context for a country. It uses the Nager.Date API to retrieve country details (borders, region) and a list of public holidays. It is read-only and context-only. No authority is granted for downstream mutations; any action based on this context requires its own receipt and authority gate.

## What this skill does

`country-holidays` performs two steps:
1.  Retrieves basic country information such as common name, official name, region, and bordering countries.
2.  Retrieves the full list of public holidays for the specified country and year.

It returns typed packets for both the country info and the holiday list, enabling downstream agents to avoid scheduling conflicts or adapt content to local contexts.

## When to use this skill

- An agent needs to check if a specific date is a public holiday in a target country.
- A workflow requires country metadata (e.g., bordering countries) for geographic reasoning.
- Planning international events or logistics where local holidays matter.

## When not to use this skill

- For real-time travel alerts or safety warnings.
- To perform actions like booking or rescheduling without a downstream action gate.
- When high-precision legal or religious holiday definitions are required beyond the public API's scope.

## Procedure

1. Identify the target `countryCode` (ISO 3166-1 alpha-2) and the `year`.
2. Fetch country information to confirm the country's identity and region.
3. Fetch the public holiday list for the specified year.
4. Normalize the data into typed packets.
5. Return the consolidated context.

## Edge cases and stop conditions

- **Invalid Country Code:** The API will return an error or empty result; the skill should return `failure` or `needs_input`.
- **Year Out of Range:** The API might not have data for very distant future/past years.
- **Rate Limiting:** If the API returns 429, the skill should report a retryable failure.
- **Timeout:** If the API is unreachable, report a timeout.

## Output schema

```yaml
country_info_packet:
  commonName: string
  officialName: string
  countryCode: string
  region: string
  borders: array
public_holidays_packet:
  holidays:
    - date: string
      localName: string
      name: string
      countryCode: string
      fixed: boolean
      global: boolean
      counties: array
      launchYear: integer
      types: array
```

## Inputs

- `countryCode` (required): Two-letter country code (ISO 3166-1 alpha-2), e.g., "US", "TR", "DE".
- `year` (required): The year to fetch holidays for, e.g., 2026.
