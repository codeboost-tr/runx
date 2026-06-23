---
name: dunning-ladder
description: Read an overdue receivable and a cadence policy, decide the next reminder step within the cap, and emit a gated reminder-send proposal, escalating once the cadence cap is reached.
runx:
  category: payments
---

# Dunning & AR Reminder Ladder

Accounts-receivable dunning needs a capped, governed cadence, not unbounded nagging. This skill reads an overdue receivable and a cadence policy, decides the next reminder step within the cap, and emits a gated reminder-send proposal, escalating once the cadence cap is reached. The send-as catalog skill performs the gated reminder.

## When to use

- A receivable is overdue and a dunning step needs to be selected.
- The sender wants to enforce a cap on total reminders before escalation.
- A governed, auditable reminder trail is required.

## When not to use

- The receivable is not actually overdue (aging_days <= 0).
- The cap has already been reached and escalation has already occurred.
- Direct, ungoverned communication is acceptable.

## Procedure

1. Read invoice_status, aging_days, and cadence_policy.
2. If aging_days <= 0, refuse: the record is not overdue.
3. If the current step index exceeds the cadence cap, escalate with no reminder.
4. Otherwise, compute the next step within the cap and emit a reminder_proposal.
5. The reminder_proposal is a gated Effect for the send-as catalog skill; this skill sends nothing itself.

## Edge cases

- aging_days <= 0: not overdue, return refused.
- step index >= cap: escalate, no further reminder.
- Partial or missing policy: return refused with clear reason.

## Output schema

```yaml
decision:
  step: int              # Current step index (0-based)
  action: string         # "remind" | "escalate" | "refused"
reminder_proposal:       # null when action is not "remind"
  channel: string        # "email" | "sms" | "letter"
  content_digest: string # SHA256 of the proposed reminder content
escalation:              # null when action is not "escalate"
  reason: string         # Why escalation was triggered
  recommended_action: string
```

## Inputs

- `invoice_status` (string, required): Current status of the invoice (e.g. "overdue", "paid", "pending").
- `aging_days` (number, required): Number of days the invoice is overdue.
- `cadence_policy` (object, required): `{ "steps": [{"max_days": number, "channel": string}], "cap": number }`.
