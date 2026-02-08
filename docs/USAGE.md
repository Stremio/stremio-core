# Stremio Core Usage Guide

This guide covers essential patterns for integrating with stremio-core.

## Getting Started

### 1. Initialize the Runtime

```rust
use stremio_core::runtime::{Runtime, RuntimeAction, RuntimeEvent};

// Create runtime with initial model and effects
let (runtime, rx) = Runtime::<MyEnv, Model>::new(model, initial_effects, 1000);

// IMPORTANT: Listen for events immediately
tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
        match event {
            RuntimeEvent::NewState(fields, ..) => { /* update UI */ }
            RuntimeEvent::CoreEvent(event) => { /* handle events */ }
        }
    }
});
```

### 2. Load Context First

**Critical**: Always load context before rendering or dispatching other actions.

```rust
// This MUST be dispatched first
runtime.dispatch(RuntimeAction {
    field: None,
    action: Action::Ctx(ActionCtx::PullUserFromAPI { token: None }),
});

// Wait until ctx.is_loaded is true before rendering
```

### 3. Load Library

```rust
// Sync library with API (fetches user's library items)
runtime.dispatch(RuntimeAction {
    field: None,
    action: Action::Ctx(ActionCtx::SyncLibraryWithAPI),
});
```

### 4. Load Notifications

```rust
// Load notifications (used globally, e.g., on detail pages)
runtime.dispatch(RuntimeAction {
    field: None,
    action: Action::Ctx(ActionCtx::PullNotifications),
});
```

## Loading Models

Use `ActionLoad` to load specific views/models:

```rust
// Load catalog with filters
runtime.dispatch(RuntimeAction {
    field: None,
    action: Action::Load(ActionLoad::CatalogWithFilters(Some(selected))),
});

// Load meta details (movie/series info)
runtime.dispatch(RuntimeAction {
    field: None,
    action: Action::Load(ActionLoad::MetaDetails(MetaDetailsSelected {
        meta_path: resource_path,
        stream_path: None,
        guess_stream: false,
    })),
});

// Load player
runtime.dispatch(RuntimeAction {
    field: None,
    action: Action::Load(ActionLoad::Player(Box::new(player_selected))),
});
```

## Common Patterns

### Authentication Flow

```rust
// Login
runtime.dispatch(RuntimeAction {
    field: None,
    action: Action::Ctx(ActionCtx::Authenticate(AuthRequest::Login {
        email: "user@example.com".into(),
        password: "password".into(),
        facebook: false,
    })),
});

// Logout
runtime.dispatch(RuntimeAction {
    field: None,
    action: Action::Ctx(ActionCtx::Logout),
});
```

### Library Management

```rust
// Add to library
runtime.dispatch(RuntimeAction {
    field: None,
    action: Action::Ctx(ActionCtx::AddToLibrary(meta_preview)),
});

// Mark as watched
runtime.dispatch(RuntimeAction {
    field: None,
    action: Action::Ctx(ActionCtx::LibraryItemMarkAsWatched {
        id: library_item_id,
        is_watched: true,
    }),
});
```

### Player Events

```rust
// Report time changes during playback
runtime.dispatch(RuntimeAction {
    field: None,
    action: Action::Player(ActionPlayer::TimeChanged {
        time: current_time_ms,
        duration: total_duration_ms,
        device: "web".into(),
    }),
});

// Report seek
runtime.dispatch(RuntimeAction {
    field: None,
    action: Action::Player(ActionPlayer::Seek {
        time: seek_to_ms,
        duration: total_duration_ms,
        device: "web".into(),
    }),
});

// Video ended
runtime.dispatch(RuntimeAction {
    field: None,
    action: Action::Player(ActionPlayer::Ended),
});
```

### Pagination

```rust
// Load next page in catalog
runtime.dispatch(RuntimeAction {
    field: None,
    action: Action::CatalogWithFilters(ActionCatalogWithFilters::LoadNextPage),
});

// Load next page in library
runtime.dispatch(RuntimeAction {
    field: None,
    action: Action::LibraryWithFilters(ActionLibraryWithFilters::LoadNextPage),
});
```

## Startup Sequence

The recommended startup sequence:

1. **Create Runtime** with initial model
2. **PullUserFromAPI** - Load user authentication  
3. **SyncLibraryWithAPI** - Load user's library
4. **PullNotifications** - Load notification data
5. **Load initial view** - e.g., `ActionLoad::CatalogWithFilters`

```rust
// Only render UI after ctx.is_loaded == true
if model.ctx.is_loaded {
    // Safe to render
}
```

## Unloading

When navigating away from a view:

```rust
runtime.dispatch(RuntimeAction {
    field: None,
    action: Action::Unload,
});
```

This cleans up model-specific state and cancels pending requests.
