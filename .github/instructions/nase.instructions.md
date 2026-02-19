---
applyTo: '**'
---
cargo check, not build. never create .md files unless instructed. avoid creating code with redundancies, avoid defining an excessive amount of new components or resources just for trivial things. if you do so, put them into their respective _components or _resources files. follow preexistent code and query style, avoid definying queries which conflict with each other, gather context before writing code. for non-systems standalone fn functions, if possible put them into a relevant struct's impl. 
only if adequate: in the case of many components sharing an extremely similar structure+impl (they are near-copy pastes of each other), define a macro. when you finish writing all your code, do a "redundancy cleaning pass". in the redundancy cleaning pass you read all the files you refactored and remove any leftover dead code, or redundancies.
