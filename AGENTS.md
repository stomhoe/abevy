
IMPORTANT: when implementing a SystemParam structonly cargo check crates you altered.
never create .md files unless instructed. avoid creating code with redundancies, avoid defining an excessive amount of new bevy Components or Resources (verbosity). if you do so, put them into their respective _components or _resources or _seris files. follow preexistent code style, avoid definying queries which conflict with each other. 
if a queried component has no fields, use Has<ComponentName> instead of Option<&ComponentName>, the former returns a bool directly. Try to make code easy to understand. 
Prefer let Ok/Some(...) else {continue;} over if let Ok/Some(...){}. Use
EntityHashmap/set over Hashmap/set<Entity>. use .read() to read MessageReader's received messages, not .iter(). to write messages with a MessageWriter, define mut messages: Local<Vec<MessageType>> in the system params, then call writer.write_batch(messages.drain(..)); at the end of the system. Don't forget to add imports. If you use something from a crate, add the dependency. if you change fields of a *Seri struct, fix dependent .ron files, and don't create legacy adapters for these outdated .rons, just update the rons.

For "target" parameter in error!, info!, trace!, debug!, warn! macros, put the corresponding constant from log_targets.rs. If constant is missing, define it in log_targets.rs and then add into main.rs's format string.

For freshly implemented features, add debug! prints with the correct "target:" const from log_targets.rs for the file and update main.rs to actually show the logs, so that the user can help you debug, but make these occupy a single line even if long. if spammy use trace! instead. Avoid overdoing it, put in the most important sections. Only do it if you suspect the code can be buggy and it's not straightforward

Make sure to .replicate::<T> newly added components which the client also uses in his locally running systems.

if you write a client-only system, register in .in_set(ClientSystems). If server-running only, register with .run_if(in_state(ServerState::Running)). If it's a system to run for either singleplayer/hoster, use .in_set(HostSystems)

If using Or<(T, U)> as query filter, put it before any other filter. Use bevy's hashmaps and hashsets over std ones.

if writing error!'s for a specific entity's failure, try to query its StrId instead of only printing its entity id.

IMPORTANT: try to put fn to types if possible (within an impl). If there's no relevant type, put them in a submodulename_helper.rs file.

For dealing with time, use bevy Timers, not raw floats.


if a system is purely message-driven (its logic running is dependent on a MessageReader), then you must register it in its plugin as foo_system.run_if(on_message::<BarMessage>),



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
Then, turn it into this:
use ::something::*;

IMPORTANT: In mono-queries (queries for a single component T), NEVER wrap the queried component in Option<&T> or Has<T>; with let Ok(t) else continue or .is_ok() you can handle any need to check presence.

AVOID putting conflicting queries in system params, queries must be disjoint. To fix overlapping queried components, extract said overlapping &Component into a new shared monoquery

when using cmd.entity(enti).try_remove::<SomeComponent>, use .try_remove directly on the enti. DO NOT collect the entis into a buffer to do it at the end of a system with a for loop iterating the buffer, it is unnecessary.

Also, cmd.try_insert_batch(some_collection) exists and it can be called with any iterable collection which yields (Entity, B:Into<Bundle>) tuples

EXTREMELY IMPORTANT: if you find a problem/impossibility/contradiction/strong ambiguity for a user-requested implementation/code fix, don't force a hacky code patch with an assumption. Instead, STOP GENERATING CODE and inform the user about the wall you ran into so he decides how to continue

try to use .reserve()/with_capacity() for collections in which you know how many you are going to insert as max

VERY IMPORTANT: if you got requested implicitely or explicitely to fix/generate some code, first check if you are in read mode (can't edit files), dont read/explore/generate any code, instead immediately tell the user to change your authorization level before beginning.


try to use dereference via destructure with & as soon as you can to avoid putting * each time you pass the var as an argument for a parameter which needs an owned type. specially for types which derive Copy

if using BlockedTileParamSet in a system, use its inner gpos query instead of querying for gpos in the consumer system, otherwise query conflict will hapen

IMPORTANT: when you exceed bevy's parameter count limit, put all Locals into a SystemParam called [Something]Locals, and all queries in pub struct [Something]Queries. define these structs right above the subject system.

IMPORTANT: when implementing a #[derive(SystemParam)] struct, take a look at previous implementations of SystemParam in codebase to see how lifetime specifiers are placed

NEVER ADD .chain() yourself when registering systems unless it is to specifically fix a bug.

AFTER YOUR CHANGES, MAKE SURE THAT THERE AREN'T DUPLICATED QUERIES QUERYING FOR THE SAME COMPONENT
