use gpui_kit::{AppContext, Context, Render, Window, div};
use gpui_navigation::{NavMotion, NavStackEvent, Navigator, NavigatorConfig, NavigatorError};

struct Page;

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui_kit::IntoElement {
        div()
    }
}

#[gpui_kit::test]
fn install_and_root_navigation(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| {
        assert!(Navigator::install(cx, NavigatorConfig::default()).is_ok());
    });

    let root = cx.new(|_| Page);
    let second = cx.new(|_| Page);

    cx.update(|cx| {
        let navigator = Navigator::new();
        assert!(navigator.push(root, cx).is_ok());
        assert!(
            navigator
                .push_with_motion(second, NavMotion::Immediate, cx)
                .is_ok()
        );

        assert_eq!(navigator.depth(cx).ok(), Some(2));
        assert_eq!(navigator.can_pop(cx).ok(), Some(true));
        assert_eq!(navigator.can_forward(cx).ok(), Some(false));
        assert!(navigator.current_as::<Page>(cx).ok().flatten().is_some());
    });
}

#[gpui_kit::test]
fn navigation_history_matches_navstack_semantics(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| {
        assert!(Navigator::install(cx, NavigatorConfig::default()).is_ok());
    });

    let first = cx.new(|_| Page);
    let second = cx.new(|_| Page);
    let third = cx.new(|_| Page);
    let replacement = cx.new(|_| Page);
    let reset_root = cx.new(|_| Page);

    cx.update(|cx| {
        let navigator = Navigator::new().immediate();

        assert!(navigator.push(first, cx).is_ok());
        assert!(navigator.push(second, cx).is_ok());
        assert!(navigator.push(third, cx).is_ok());
        assert_eq!(navigator.depth(cx).ok(), Some(3));

        assert!(navigator.pop(cx).ok().flatten().is_some());
        assert_eq!(navigator.depth(cx).ok(), Some(2));
        assert_eq!(navigator.can_forward(cx).ok(), Some(true));

        assert!(navigator.forward(cx).ok().flatten().is_some());
        assert_eq!(navigator.depth(cx).ok(), Some(3));
        assert_eq!(navigator.can_forward(cx).ok(), Some(false));

        assert!(navigator.pop(cx).ok().flatten().is_some());
        assert!(navigator.push(replacement, cx).is_ok());
        assert_eq!(navigator.can_forward(cx).ok(), Some(false));
        assert_eq!(navigator.depth(cx).ok(), Some(3));

        assert!(navigator.replace(reset_root, cx).ok().flatten().is_some());
        assert_eq!(navigator.depth(cx).ok(), Some(3));
    });
}

#[gpui_kit::test]
fn nested_scopes_are_independent(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| {
        assert!(Navigator::install(cx, NavigatorConfig::default()).is_ok());
    });

    let root = cx.new(|_| Page);
    let nested_a = cx.new(|_| Page);
    let nested_b = cx.new(|_| Page);

    cx.update(|cx| {
        let root_nav = Navigator::new();
        let nested_nav = Navigator::new().scope("workspace");
        let child_nav = nested_nav.scope("settings");

        assert!(root_nav.push(root, cx).is_ok());
        assert!(nested_nav.push(nested_a, cx).is_ok());
        assert!(nested_nav.push(nested_b, cx).is_ok());
        assert!(child_nav.push(cx.new(|_| Page), cx).is_ok());

        assert_eq!(root_nav.depth(cx).ok(), Some(1));
        assert_eq!(nested_nav.depth(cx).ok(), Some(2));
        assert_eq!(child_nav.depth(cx).ok(), Some(1));
        assert_eq!(nested_nav.path().to_string(), "/workspace");
        assert_eq!(child_nav.path().to_string(), "/workspace/settings");
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

    cx.update(|cx| {
        assert!(Navigator::install(cx, NavigatorConfig::default()).is_ok());
    });

    let root = cx.new(|_| Page);
    let second = cx.new(|_| Page);
    let observer = cx.new(|_| Observer {
        events: Vec::new(),
        subscription: None,
    });

    cx.update(|cx| {
        observer.update(cx, |observer, cx| {
            let subscription = Navigator::new().subscribe(cx, |observer, event, _| {
                observer.events.push(*event);
            });
            assert!(subscription.is_ok());
            observer.subscription = subscription.ok();
        });
    });

    cx.update(|cx| {
        let nav = Navigator::new();
        assert!(nav.push(root, cx).is_ok());
        assert!(nav.push(second, cx).is_ok());
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
    cx.update(|cx| {
        assert!(Navigator::install(cx, NavigatorConfig::default()).is_ok());
    });

    let page = cx.new(|_| Page);

    cx.update(|cx| {
        let scope = Navigator::new().scope("workspace");
        let child = scope.scope("settings");

        assert!(scope.push(page.clone(), cx).is_ok());
        assert!(child.push(cx.new(|_| Page), cx).is_ok());
        assert_eq!(scope.remove_scope(cx).ok(), Some(true));

        assert!(matches!(
            scope.depth(cx),
            Err(NavigatorError::ScopeNotFound(_))
        ));
        assert!(matches!(
            child.depth(cx),
            Err(NavigatorError::ScopeNotFound(_))
        ));
    });
}

#[gpui_kit::test]
fn duplicate_install_is_rejected(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| {
        assert!(Navigator::install(cx, NavigatorConfig::default()).is_ok());
        assert!(matches!(
            Navigator::install(cx, NavigatorConfig::default()),
            Err(NavigatorError::AlreadyInstalled)
        ));
    });
}
