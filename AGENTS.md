only cargo check crates you altered.
never create .md files unless instructed. avoid creating code with redundancies, avoid defining an excessive amount of new components or resources (verbosity). if you do so, put them into their respective _components or _resources or _seris files. follow preexistent code style, avoid definying queries which conflict with each other. 
if a queried component has no fields, use Has<ComponentName> instead of Option<&ComponentName>, the former returns a bool directly. Try to make code compacted. Prefer let Ok/Some(...) else {continue;} over if let Ok/Some(...){}. Use
EntityHashmap/set over Hashmap/set<Entity>. use .read() to read MessageReader's received messages, not .iter(). to write messages with a MessageWriter, define mut messages: Local<Vec<MessageType>> in the system params, then call writer.write_batch(messages.drain(..)); at the end of the system. Don't forget to add imports. If you use something from a crate, add the dependency. if you change fields of a *Seri struct, fix dependent .ron files, and don't create legacy adapters for these outdated .rons, just update the rons.

For target: in error!, info!, etc. put the corresponding constant from log_targets.rs. If constant is missing, define it in log_targets.rs and then add into main.rs's format string 

For freshly implemented features, add debug! prints and update main.rs to actually show the logs, so that the user can help debug, but make these occupy a single line even if long. if spammy use trace! instead.

Make sure to .replicate::<T> newly added components which the client also uses in his locally running systems.

if you write a client-only system, register in .in_set(ClientSystems). If server-running only, register with .run_if(in_state(ServerState::Running)). If it's a system to run for either singleplayer/hoster, use .in_set(HostSystems)

If using Or<(T, U)> as query filter, put it before any other filter. Use bevy's hashmaps and hashsets over std ones.

if writing error!'s for a specific entity's failure, try to query its StrId instead of only printing its entity id.

try to associate helper fns to types if possible, if not put them in a submodulename_helper.rs file

for dealing with time, use bevy Timers, not raw floats


if a system is purely message-driven (its logic is dependant on a MessageReader), then you must register it in its plugin as foo_system.run_if(on_message::<BarMessage>),



VERY IMPORTANT: for bevy systems, always put #[allow(unused_parens, )] on top of their definition, and for each query, ALWAYS include two sets of parentheses, one for the component querying part and for the filtering part, even if unnecessary, also leave trailing commas to the left of the right enclosing parentheses. Always like this:
Query<(&ComponentType, ), (With<ComponentType2>, )>,

Prefer imports like this: 
use somethings::*
Over imports like this:
use something::FinalNormMoveDir;
use something::InputMoveDir;
use something::SpeedMagnitude;
If you find 2 or more imported things within curly braces, like this:
use ::something::{ChunkPos, DimensionRef, GlobalTilePos};
turn it into this:
use ::something::*;

in mono-queries (queries for a single component T), NEVER wrap the queried component in Option<&T> or Has<T>; with let Ok(t) else continue or .is_ok() you can handle any need.

AVOID putting conflicting queries in system params, queries must be disjoint. To fix overlapping queried components, extract said overlapping &Component into a new shared monoquery

if you find a problem, stop generating code instead of implementing a hacky solution, tell the user for
