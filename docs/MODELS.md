# Model Writing Policy

Guidelines for writing and maintaining models in stremio-core.

## Core Principles

### 1. Business Logic Lives in Models

**All business logic should be handled entirely in the model**, especially:

- Mutability and state transitions
- Data validation and transformation
- Computed properties and derived state

The more logic we have in models, the less chance for logic mistakes in view code.

```rust
// ✅ GOOD: Logic in model
impl Model {
    pub fn is_notification(&self, video: &Video) -> bool {
        // All notification logic here
        self.compute_notification_status(video)
    }
}

// ❌ BAD: Logic in view
// View code manually checking notification conditions
```

### 2. Avoid Borderline Properties

Be cautious with properties that combine model data:

```rust
// ⚠️ BORDERLINE: Combined loading state
// Different views may want separate loaders per group
pub fn is_still_loading(&self) -> bool {
    self.groups.iter().any(|g| g.is_loading)
}
```

If different views might need different behaviors, let them implement the logic.

### 3. Include Related Data from Context

When model data needs context data, include it in the model even if it means extra copying:

```rust
// ✅ GOOD: Pair catalog items with library info
pub struct CatalogItem {
    pub meta: MetaItemPreview,
    pub library_item: Option<LibraryItem>, // Retrieved from Ctx
}

// ❌ BAD: Force view to look up library items
pub struct CatalogItem {
    pub meta: MetaItemPreview,
    // View must call ctx.library.get(meta.id) - error prone!
}
```

This prevents bugs from views forgetting to look up related data.

## Effects Guidelines

### Effect Types

```rust
// Synchronous - immediate message
Effect::Msg(Box::new(msg))

// Concurrent - runs in parallel
Effect::Future(EffectFuture::Concurrent(future))

// Sequential - waits for previous sequential effects
Effect::Future(EffectFuture::Sequential(future))
```

### Looping Messages Back

To dispatch a message from within an effect:

```rust
use futures::future::ok;

// Loop a message back into the system
Effects::one(Effect::Future(EffectFuture::Concurrent(
    Box::pin(ok(Msg::Internal(internal_msg)))
)))

// Or use the shorthand
Effects::msg(Msg::Internal(internal_msg))
```

### Effect Return Values

```rust
// Changed state, no side effects
Effects::none()

// Changed state with effects
Effects::one(effect)
Effects::many(effects)

// Unchanged state (no re-render trigger)
Effects::none().unchanged()
Effects::default() // Same as none().unchanged()
```

## Model Update Pattern

```rust
impl<E: Env + 'static> UpdateWithCtx<E> for MyModel {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::MyAction(action)) => {
                // 1. Update state
                self.handle_action(action);
                
                // 2. Return effects
                Effects::none() // or with effects
            }
            Msg::Internal(Internal::SomeResponse(data)) => {
                // Handle internal messages from effects
                self.process_response(data);
                Effects::none()
            }
            _ => Effects::default(), // Unchanged
        }
    }
}
```

## Best Practices

### Do

- Keep all state mutation in `update()`
- Return `Effects::default()` for unhandled messages
- Use `Effects::none().unchanged()` when state doesn't change
- Include related context data in model structs
- Document complex business logic

### Don't

- Mutate state outside of `update()`
- Have effects resolve to `Action` messages (will panic)
- Force views to look up related data from context
- Create borderline helper properties that limit view flexibility

## Testing

Models should be tested in isolation:

```rust
#[test]
fn test_model_updates() {
    let mut model = MyModel::default();
    let ctx = test_ctx();
    
    // Test action handling
    let effects = model.update(&Msg::Action(my_action), &ctx);
    
    assert!(effects.has_changed);
    assert_eq!(model.expected_state, expected_value);
}
```
