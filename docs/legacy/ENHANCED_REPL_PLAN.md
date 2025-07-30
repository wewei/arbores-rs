# Enhanced REPL Implementation Plan

## Dependencies to Add

```toml
[dependencies]
reedline-repl-rs = "1.2"
clap = { version = "4.0", features = ["derive"] }
ctrlc = "3.4" # Already present
```

## New Module Structure

```
src/repl/
├── mod.rs          # Current basic REPL
├── enhanced.rs     # New enhanced REPL using reedline-repl-rs
└── common.rs       # Shared utilities
```

## Implementation Steps

### Step 1: Add Dependencies

First, we need to add the required dependencies to Cargo.toml.

### Step 2: Create Enhanced REPL Module

Create a new enhanced REPL that provides:
- Command history
- Syntax highlighting for Scheme
- Auto-completion for built-in functions
- Multi-line input support
- Better error display

### Step 3: Command Line Interface

Add CLI options to choose between REPL modes:
- `--repl=simple` for current basic REPL
- `--repl=enhanced` for new enhanced REPL (default)

### Step 4: Integration

Integrate the enhanced REPL while keeping the simple one as fallback.

## Features to Implement

### Basic Features (Phase 1)
- [x] Command history with persistence
- [x] Basic syntax highlighting
- [x] Multi-line input for complex expressions
- [x] Better error display

### Advanced Features (Phase 2)
- [ ] Auto-completion for built-in functions
- [ ] Bracket matching and highlighting
- [ ] Variable and function name completion
- [ ] Help system integration

### Expert Features (Phase 3)
- [ ] Debugging support (breakpoints, step execution)
- [ ] Expression evaluation preview
- [ ] Documentation lookup
- [ ] Custom themes

## Code Example

Here's how the enhanced REPL will be structured:

```rust
use reedline_repl_rs::*;
use crate::eval::Evaluator;

#[derive(clap::Parser)]
#[command(name = "arbores")]
#[command(about = "Enhanced Scheme REPL")]
struct Args {
    #[clap(long, default_value = "scheme")]
    context: String,
}

struct ArborRepl {
    evaluator: Evaluator,
}

impl Repl for ArborRepl {
    type Item = String;
    type Context = String;

    fn eval(&mut self, command: Self::Item, context: &mut Self::Context) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match self.evaluator.eval_string(&command) {
            Ok(result) => Ok(Some(result.to_string())),
            Err(error) => Err(Box::new(error)),
        }
    }
}
```

This implementation will provide a much better user experience while maintaining compatibility with the existing codebase.
