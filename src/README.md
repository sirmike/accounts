# Assumptions

I use csv reader to read transactions file sequentially, this saves a lot of memory.
But there is a drawback, we don't have transaction values for disputed/resolved/backcharged transactions.

I assumed several things:
* I don't need to know details about resolve/chargeback as long as we know details of a dispute. For resolves / chargebacks I can use the same transaction data as for disputes.
* Disputes number should be much lower than total transactions number so I decided to build a in-memory cache for them. Although, it may be not enough and will require further optimisations.
* Reading files is relatively fast task so it may be worth the shot to do it twice to save a lot of memory. Especially when we have fast disks (which should be true in production).
* If there is no dispute, there is no sense to try to resolve or chargeback a transaction.

# Implementation details

* I use decimal crate to avoid floating point precision errors.
* There are a few places with `unwrap()` method, just for simplicity. I assumed that it should be safe enough to use it.
