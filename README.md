# Validus Backend Engineer Exercise

## Overview

The aim of this exercise is to develop a trade approval system, enabling users to submit trade details for approval per some established protocol, and for our system to support various workflows given their trade details.

## Users

In this workflow, there are 2 types of users with different access levels.

- Requesters, who can make trade requests.
- Approvers, who are the arbiters of each trade request.

### Requesters

Requester users can:

- Submit a new trade request.
- Cancel their trade requests.
- Approve updates to their trade request details by an Approver.
- Request updates to their trade request details.
- Book trades into the system once executed by the 3rd party.

These actions are labelled as `REQUEST_NEW, APPROVE_UPDATE, CANCEL, REQUEST_UPDATE, BOOK`.

### Approver

Approver users can:

- Approve new trade requests.
- Cancel other's trade requests.
- Send approved trades to the counterparty for execution.

These actions are labelled as `APPROVE_NEW, CANCEL, SEND_TO_EXECUTE, BOOK`.

## Trade Detail Format

- `trading_entity` ~ Legal entity conducting the trade.
- `counterparty` ~ Legal entity on the other side of the trade.
- `direction` ~ Buy or Sell.
- `style` ~ "Assumes the trade is a forward contract."
- `notional_currency` ~ IBAN currency code.
- `notional_amount` ~ Size of the trade in the `notional_currency`.
- `underlying` ~ List of eligible notional currencies; which the `notional_currency` must be a member of.
- `trade_date` ~ Date when the trade is initiated.
- `value_date` ~ Date when the trade value is realized.
- `delivery_date` ~ Date when assets are delivered.
- `strike` ~ Agreed rate of the trade, only available after trades are executed.

Note that `trade_date` < `value_date` < `delivery_date`.

## State Transitions

Trades can transition between states.

- `DRAFT` ~ Trade has been created but not submitted. Permitted next actions: `[SUBMIT]`.
- `PENDING_APPROVAL` ~ Trade has been submitted but is waiting approval. Permitted next actions: `[ACCEPT, CANCEL]`
- `NEEDS_REAPPROVAL` ~ Trade details were updated by an Approver, requiring reapproval from the original requester. Permitted next actions: `[APPROVE, CANCEL]`.
- `APPROVED` ~ Trade has been approved and is ready to be sent. Permitted next actions: `[SEND_TO_EXECUTE, CANCEL]`.
- `SEND_TO_COUNTERPARTY` ~ Trade has been sent to the counterparty. Permitted next actions: `[BOOK, CANCEL]`.
- `EXECUTED` ~ Trade has been executed. This is an end state, and it's implied all `BOOK` actions lead to this final state.
- `CANCELLED` ~ Trade has been cancelled. This is an end state, and it's implied all `CANCEL` actions lead to this final state.

## Requirements

- All data is stored in memory, if the process dies all information will be lost.
- This API is written in Rust.
- Validation rules and trade details are enforced.
- Users are also able to make requests to view history of trades, including:
  - Trade details at any previous state.
  - Tabular history of actions with user IDs, timestamps and state transitions.
  - Differences to versions of trade details.
- Trades can be sent to a counterparty and marked as executed.

Other potential features we can do with this project:

- Defining a gRPC API to expose this as a service.
- Let users revert some changes instead of just cancelling. Note this will have some authorization constraints.
