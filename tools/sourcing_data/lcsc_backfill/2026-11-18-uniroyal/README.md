# UNI-ROYAL identity confirmation, 2026-11-18

The alias and size/power grouping are recorded in [issue #125](https://github.com/conol-ai/cohdl/issues/125).
The inputs are the library at aad42df and 289 exact UNI-ROYAL MPNs. To reconstruct the latter, take distinct MPNs from the `alternates`, `zero_ohm` and `openwalkietalkie_rev_b` arrays in `tools/passive_data/uniroyal_dry_run.json`. Add one AVL block per MPN to a temporary copy of the library, with manufacturer `UNI-ROYAL`. The 288 B1 rows and the extended Rev B order line C25760 all matched the expected C number.

The ordinary B2 backfill used all candidate pages for each identity under the reviewed alias table. Uncached fetches were restricted to C14661, C145168, C51801, C145170, C146168, C149502, C38522, C52199, C67088 and C30648. The client made 11 requests including robots.txt. As in the previous backfill, the unrelated SGMICRO candidate C183309 stayed outside that list and in review. No new UNI-ROYAL identity is pending.

All 12,327 earlier identities retain their C number, category and match rule. Promotion yields 12,616 rows. The candidate registry catalog gains C25760; the other 288 rows overlap B1. To rebuild the catalog offline run:

```sh
python3 -I tools/upload_supplier_snapshot.py build -o OUT --dates tools/sourcing_data/registry_uploads/2026-11-05-lcsc/report.json
```

Expected 13,605 rows, 3,584,171 bytes, SHA-256 `5b343a8e524bfed64475dee919a0b84a61a373d7a84e13e522fa8eac2f01d17b`. The serialized aliases have SHA-256 `d2d45b701ba08906383c480615d4b746d561a66f6789246e294b32988b48f068`. Production upload requires a separate sign-off.
