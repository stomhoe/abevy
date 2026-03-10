do cargo check, not build, and only cargo check crates you altered. but dont compile if you only changed non-.rs files
never create .md files unless instructed. avoid creating code with redundancies, avoid defining an excessive amount of new components or resources. if you do so, put them into their respective _components or _resources files. follow preexistent code and query style, avoid definying queries which conflict with each other. 
if a queried component has no fields, use Has<ComponentName> instead of Option<&ComponentName>, the former returns a bool directly. Try to make code compacted. Prefer let Ok/Some(...) else {continue;} over if let Ok/Some(...){}. prefer
EntityHashmap/set over Hashmap/set<Entity>. use .read() to read MessageReader's received messages, not .iter(). to write messages with a MessageWriter, define a mut messages: Local<Vec<MessageType>> in the system params, then use writer.write_batch(messages.drain(..)); at the end of the system. don't forget to add imports. if you use something from a crate, add the dependency. if you alter a *Seri, fix affected .ron files

for target: in error!, info!, etc. put the corresponding constant from log_targets.rs. if constant is missing, add it in there and then into main.rs's


for freshly implemented features, add debug! prints and update main.rs to actually show the logs, so that the user can give you feedback. for systems which are ran both by server and clients, make sure that you .replicate::<ComponentName>'d all involved components in the corresponding module's pub fn plugin. make sure to replicate tile related components.

if you write a client-only system, put .in_set(ClientSystems). if serveropened-only, put .run_if(in_state(ServerState::Running)). if either for singleplayer/host, .in_set(HostSystems)
