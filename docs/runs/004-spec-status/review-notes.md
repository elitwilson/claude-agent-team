# 004-spec-status Review Notes

## Task: Frontmatter parsing in spec discovery — RED review

**Reviewer verdict: APPROVED**

All spec requirements have corresponding test coverage:

| Spec Requirement | Test(s) |
|---|---|
| Parse `status: ready` | `test_parse_frontmatter_status_ready` |
| Parse `status: complete` | `test_parse_frontmatter_status_complete` |
| Parse `status: needs_attention` | `test_parse_frontmatter_status_needs_attention` |
| Filter out `complete` specs | `test_discover_specs_filters_out_complete` |
| Include `needs_attention` specs | `test_discover_specs_includes_needs_attention` |
| Missing frontmatter defaults to `ready` | `test_parse_frontmatter_status_missing_frontmatter` + `test_discover_specs_treats_no_frontmatter_as_ready` |
| Missing status field defaults to `ready` | `test_parse_frontmatter_status_missing_status_field` |
| Unrecognized status defaults to `ready` | `test_parse_frontmatter_status_unrecognized_value` |
| Empty frontmatter defaults to `ready` | `test_parse_frontmatter_status_empty_frontmatter` |
| Return type carries status for TUI | `SpecEntry` struct with `name` + `status`, verified in filtering tests |

No issues found. Tests cover observable behavior, not implementation details. Existing tests updated correctly for the new return type.

## Task: TUI yellow highlight — RED review

**Reviewer verdict: APPROVED**

| Spec Requirement | Test(s) | Notes |
|---|---|---|
| Status flows through to render layer | `test_app_carries_spec_status` | Verifies `App.specs` carries `SpecStatus` |
| `TuiResult.spec` remains a String | `test_result_returns_spec_name_not_entry` | Downstream compat preserved |
| `needs_attention` specs navigable/selectable | `test_needs_attention_spec_is_navigable` | Can select and read status |
| Yellow styling for `needs_attention` | Not unit-tested | Correct — testing `ratatui` `Style` on `ListItem` would test implementation details, not behavior. The contract is that status reaches the render layer, which is covered. |

No issues found. The Coder made the right call not testing the exact `Color::Yellow` style — that would couple tests to ratatui rendering internals. The tests enforce the actual contract: status data is available at the point where rendering decisions are made.
