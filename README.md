# Pay Engine

A simple toy payments engine that reads a series of transactions from a CSV, updates client accounts, handles disputes and chargebacks, and then outputs the
state of clients accounts as a CSV.
## Transactions

Transactions will come in this form:
```text
type,       client, tx, amount
deposit,    5,      1,  42.2
deposit,    2,      2,  2.0
deposit,    3,      3,  2.0
withdrawal, 5,      5,  1.5
withdrawal, 2,      4,  3.0
dispute,    1,      1
chargeback, 1,      1
resolve,    1,      1
resolve,    1,      1
resolve,    1,      1
deposit,    2,      7,  3.8
```

## AccountsRepository

A AccountsRepository tracks clients accounts.

A Client has his funds split into two balances:
- `available` funds, which are ready to be used in transactions
- `held` funds, that are involved in disputed transactions (see below)

available|held|total
---------|----|-----
f64|f64|f64

## Types of operations

There are 5 kind of transactions:

### **Deposit**

A deposit is a credit to the client's asset account, meaning it should increase the available and
total funds of the client account.

### **Withdrawal**

A deposit is a credit to the client's asset account, meaning it should increase the available and
total funds of the client account.

### **Dispute**

A dispute represents a client's claim that a transaction was erroneous and should be reversed.
The transaction shouldn't be reversed yet but the associated funds should be held. This means
that the clients available funds should decrease by the amount disputed, their held funds should
increase by the amount disputed, while their total funds should remain the same.

### **Resolve**

A resolve represents a resolution to a dispute, releasing the associated held funds. Funds that
were previously disputed are no longer disputed. This means that the clients held funds should
decrease by the amount no longer disputed, their available funds should increase by the
amount no longer disputed, and their total funds should remain the same.

### **Chargeback**

A chargeback is the final state of a dispute and represents the client reversing a transaction.
Funds that were held have now been withdrawn. This means that the clients held funds and
total funds should decrease by the amount previously disputed. If a chargeback occurs the
client's account should be immediately frozen.

# Building and Running

The project can be run against input CSV file if you have predefined scenarios to run.

```bash
cargo run -q -- file_path.csv
```

# Testing

In order to run e2e tests run:

```bash
make test/e2e
```

In order to run unit tests run:

```bash
make test/unit
```

# Improvements

- [ ] Add channel to enable streaming values through memory: consumer producer pattern.