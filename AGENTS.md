only cargo check crates you altered.
never create .md files unless instructed. avoid creating code with redundancies, avoid defining an excessive amount of new components or resources. if you do so, put them into their respective _components or _resources or _seris files. follow preexistent code and query style, avoid definying queries which conflict with each other. 
if a queried component has no fields, use Has<ComponentName> instead of Option<&ComponentName>, the former returns a bool directly. Try to make code compacted. Prefer let Ok/Some(...) else {continue;} over if let Ok/Some(...){}. prefer
EntityHashmap/set over Hashmap/set<Entity>. use .read() to read MessageReader's received messages, not .iter(). to write messages with a MessageWriter, define mut messages: Local<Vec<MessageType>> in the system params, then call writer.write_batch(messages.drain(..)); at the end of the system. Don't forget to add imports. If you use something from a crate, add the dependency. if you change fields of a *Seri struct, fix dependent .ron files

For target: in error!, info!, etc. put the corresponding constant from log_targets.rs. If constant is missing, define it in log_targets.rs and then add into main.rs's format string 

For freshly implemented features, add debug! prints and update main.rs to actually show the logs, so that the user can help debug, but make these occupy a single line even if long. if spammy use trace! instead. For systems which are ran both by server and clients, make sure that you .replicate::<ComponentName>'d all involved components in the corresponding module's pub fn plugin. Make sure to .replicate::<T> newly added components which the client also uses in his locally running systems.

if you write a client-only system, put .in_set(ClientSystems). If server-running only, put .run_if(in_state(ServerState::Running)). If it's a system for either singleplayer/hosting, use .in_set(HostSystems)

if using Or<(T, U)> as query filter, put it before any other filter. use bevy's hashmaps and hashsets over std ones.
