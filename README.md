# gpui-navigation

Typed hierarchical navigation for GPUI desktop applications.

Designed for `gpui-kit 0.6.1`.

## Consumer dependency

```toml
gpui-navigation = "0.1"
```

The proc-macro crate is re-exported by `gpui-navigation`, so applications do not need to depend on it directly.

## Route declaration

```rust
use gpui_navigation::RouteSegment;

#[derive(RouteSegment, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectsRoute {
    List,
    Open(u64),
}
```

## Navigation

```rust
Navigator::replace(ProjectsRoute::Open(42), cx);
Navigator::push(ProjectRoute::Firmware, cx);
Navigator::back(cx);
Navigator::up(cx);
```

## Absolute path

```rust
let path = NavPath::root(WorkspaceRoute::Projects)
    .push(ProjectsRoute::Open(42))
    .push(ProjectRoute::Firmware);

Navigator::go(path, cx);
```

`Navigator::back` uses history. `Navigator::up` follows the structural parent of the current path.
