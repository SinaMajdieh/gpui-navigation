use gpui_kit::{AppContext, Context, Render, Window, div};
use gpui_navigation::{NavMotion, NavStackEvent, Navigator, NavigatorConfig, NavigatorError};

#[derive(Clone, Copy, Debug)]
struct Workspace;

#[derive(Clone, Copy, Debug)]
struct Settings;

struct Page;

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui_kit::IntoElement {
        div()
    }
}

#[gpui_kit::test]
fn install_and_root_navigation(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| Navigator::install(cx, NavigatorConfig::default()));

    let root = cx.new(|_| Page);
    let second = cx.new(|_| Page);

    cx.update(|cx| {
        let navigator = Navigator::new();
        navigator.push(root, cx);
        navigator.push_with_motion(second, NavMotion::Immediate, cx);

        assert_eq!(navigator.depth(cx), 2);
        assert!(navigator.can_pop(cx));
        assert!(!navigator.can_forward(cx));
        assert!(navigator.current_as::<Page>(cx).is_some());
    });
}

#[gpui_kit::test]
fn fallible_api_remains_available(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| {
        Navigator::try_install(cx, NavigatorConfig::default()).unwrap();

        let root = cx.new(|_| Page);
        Navigator::new().try_push(root, cx).unwrap();
        assert_eq!(Navigator::new().try_depth(cx).unwrap(), 1);
        Navigator::new().try_back(cx).unwrap();
    });
}

#[gpui_kit::test]
fn typed_scopes_are_independent(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| Navigator::install(cx, NavigatorConfig::default()));

    let root = cx.new(|_| Page);
    let workspace_a = cx.new(|_| Page);
    let workspace_b = cx.new(|_| Page);
    let settings = cx.new(|_| Page);

    cx.update(|cx| {
        let root_nav = Navigator::new();
        let workspace = root_nav.scope(Workspace);
        let nested = workspace.scope(Settings);

        root_nav.push(root, cx);
        workspace.push(workspace_a, cx);
        workspace.push(workspace_b, cx);
        nested.push(settings, cx);

        assert_eq!(root_nav.depth(cx), 1);
        assert_eq!(workspace.depth(cx), 2);
        assert_eq!(nested.depth(cx), 1);
        assert_eq!(workspace.path().unwrap().depth(), 1);
        assert_eq!(
            workspace
                .path()
                .unwrap()
                .segments()
                .next()
                .unwrap()
                .type_id(),
            std::any::TypeId::of::<Workspace>()
        );

        assert_eq!(Navigator::current_scope_path(cx).unwrap().depth(), 2);
        assert_eq!(Navigator::new().current_scope().depth(cx), 1);
    });
}

#[gpui_kit::test]
fn current_scope_navigator_targets_latest_active_scope(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| Navigator::install(cx, NavigatorConfig::default()));

    let root = cx.new(|_| Page);
    let workspace = cx.new(|_| Page);
    let workspace_second = cx.new(|_| Page);

    cx.update(|cx| {
        Navigator::new().push(root, cx);

        let workspace_nav = Navigator::new().scope(Workspace).immediate();
        workspace_nav.push(workspace, cx);
        workspace_nav.push(workspace_second, cx);

        let current = Navigator::new().current_scope().immediate();
        assert_eq!(current.depth(cx), 2);
        assert!(current.can_pop(cx));
        current.back(cx);
        assert_eq!(workspace_nav.depth(cx), 1);
    });
}

#[gpui_kit::test]
fn current_scope_can_append_typed_children(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| Navigator::install(cx, NavigatorConfig::default()));

    let workspace = cx.new(|_| Page);
    let settings = cx.new(|_| Page);

    cx.update(|cx| {
        let workspace_nav = Navigator::new().scope(Workspace).immediate();
        workspace_nav.push(workspace, cx);

        let current_child = Navigator::new().current_scope().scope(Settings);
        current_child.push(settings, cx);

        assert_eq!(workspace_nav.depth(cx), 1);
        assert_eq!(current_child.depth(cx), 1);
    });
}

#[gpui_kit::test]
fn navigation_history_matches_navstack_semantics(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| Navigator::install(cx, NavigatorConfig::default()));

    let first = cx.new(|_| Page);
    let second = cx.new(|_| Page);
    let third = cx.new(|_| Page);
    let replacement = cx.new(|_| Page);
    let reset_root = cx.new(|_| Page);

    cx.update(|cx| {
        let navigator = Navigator::new().immediate();

        navigator.push(first, cx);
        navigator.push(second, cx);
        navigator.push(third, cx);
        assert_eq!(navigator.depth(cx), 3);

        navigator.pop(cx);
        assert_eq!(navigator.depth(cx), 2);
        assert!(navigator.can_forward(cx));

        navigator.forward(cx);
        assert_eq!(navigator.depth(cx), 3);
        assert!(!navigator.can_forward(cx));

        navigator.pop(cx);
        navigator.push(replacement, cx);
        assert!(!navigator.can_forward(cx));
        assert_eq!(navigator.depth(cx), 3);

        navigator.replace(reset_root, cx);
        assert_eq!(navigator.depth(cx), 3);
    });
}

#[gpui_kit::test]
fn subscription_receives_navigation_events(cx: &mut gpui_kit::TestAppContext) {
    struct Observer {
        events: Vec<NavStackEvent>,
        subscription: Option<gpui_kit::Subscription>,
    }

    impl Render for Observer {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui_kit::IntoElement {
            div()
        }
    }

    cx.update(|cx| Navigator::install(cx, NavigatorConfig::default()));

    let root = cx.new(|_| Page);
    let second = cx.new(|_| Page);
    let observer = cx.new(|_| Observer {
        events: Vec::new(),
        subscription: None,
    });

    cx.update(|cx| {
        observer.update(cx, |observer, cx| {
            observer.subscription = Navigator::new().subscribe(cx, |observer, event, _| {
                observer.events.push(*event);
            });
        });
    });

    cx.update(|cx| {
        Navigator::new().push(root, cx);
        Navigator::new().push(second, cx);
    });

    cx.read(|cx| {
        let observer = observer.read(cx);
        assert_eq!(
            observer.events,
            vec![NavStackEvent::Pushed, NavStackEvent::Pushed]
        );
    });
}

#[gpui_kit::test]
fn scope_removal_releases_scope_and_descendants(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| Navigator::install(cx, NavigatorConfig::default()));

    let page = cx.new(|_| Page);

    cx.update(|cx| {
        let scope = Navigator::new().scope(Workspace);
        let child = scope.scope(Settings);

        scope.push(page.clone(), cx);
        child.push(cx.new(|_| Page), cx);
        assert!(scope.remove_scope(cx));

        assert!(matches!(
            scope.try_depth(cx),
            Err(NavigatorError::ScopeNotFound(_))
        ));
        assert!(matches!(
            child.try_depth(cx),
            Err(NavigatorError::ScopeNotFound(_))
        ));
        assert_eq!(Navigator::try_current_scope_path(cx).unwrap().depth(), 0);
    });
}

#[gpui_kit::test]
fn duplicate_install_is_reported_without_panicking(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| {
        Navigator::install(cx, NavigatorConfig::default());
        Navigator::install(cx, NavigatorConfig::default());
        assert!(Navigator::is_installed(cx));
    });
}

#[gpui_kit::test]
fn non_fallible_calls_are_safe_before_install(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| {
        let navigator = Navigator::new();
        navigator.back(cx);
        navigator.forward(cx);
        navigator.clear(cx);
        assert_eq!(navigator.depth(cx), 0);
        assert!(navigator.is_empty(cx));
        assert!(!navigator.can_pop(cx));
        assert!(!navigator.can_forward(cx));
        assert!(navigator.is_at_root(cx));
        assert!(navigator.current(cx).is_none());
        assert!(navigator.views(cx).is_empty());
        assert!(Navigator::current_scope_path(cx).is_none());
    });
}
