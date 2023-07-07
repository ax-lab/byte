use super::*;

/*
	the machine has a heap of operators sorted by precedence

	the machine is fed a tree of nodes to operate with

	the machine will evaluate its operators in groups sorted by precedence

	in each group, the machine will ask the operators to evaluate the tree
	of nodes

	each operator will evaluate the tree of nodes and return "no change" or
	a set of changes (including eventual evaluation errors)

	if all operators in the group return "no change", the machine will evaluate
	the next group of operators

	if all groups of operators return "no change", then the evaluation is
	complete

	if a set of changes is returned, the machine will merge all changes,
	generating a new tree of nodes

	if there's any ambiguity in the set of changes, then the machine will
	report an error

	the tree is the entire program, including all file tree, evaluation nodes,
	absolutely everything, all evaluated at once in a big pile of stuff that
	must be evaluated fast

	as the tree is mutated, newly added nodes, will get picked up by operators
	higher in the precedence list

	at the end of the process, the node tree will spit code to be executed


	how to make that thing fast?
	===========================

	instead of managing their children directly, nodes create node lists
	that are managed by the compiler

	operators evaluate all node lists at once and generate a set of changes
	applying to everything

	the new hierarchy of nodes means some node lists will be discarded, while
	new ones are added to the set

	the paramount thing is being able to quickly index nodes of interest for
	an operator in the set of all lists of nodes

		operators need to quickly find all segments that can be affected by
		then and then quickly find interest nodes inside those segments

	how would you add cache into that thing?
	=======================================

	with an initial set of operators and an initial set of nodes, you can
	always cache the final result

	the above also applies to subtrees, so you could always isolate a subtree
	and a set of operators and cache that

	the cache would be invalidated if an operator is defined in a part of the
	tree that bubbles up to the root, causing the set of operators to change

	in the extreme, if there's global dependencies between the entire tree then
	the cache would be extremely fragile to any change in child nodes

	a new operator defined from a changed subtree would "bubble up" to the root
	of the tree and invalidate all caches... the entire tree would need to
	be reevaluated from scratch, since a new operator could introduce
	unpredictable changes in the previously cached nodes that could feed back
	into the already evaluated changed parts in an earlier part of the process

	STRONG SCOPING RULES ARE PARAMOUNT FOR CACHING

	the cache mechanism could still happen, and also the invalidation mentioned
	above, but now changes in subtrees would be contained, preventing the entire
	thing from be invalidated

	we still want to avoid re-evaluating from scratch at all costs, so the
	caching should employ strong guarantees that cached nodes are not
	affected by the changed parts -> if they are, cache invalidation MUST BE
	EARLY IN THE PROCESS

	we can also use SPAN TRACKING to track nodes that are dependent on each
	other and use that to invalidate all cache

	how to deal with scopes and such
	================================

	operators such as `let`, `const`, `fn`, `define`, etc., need access to
	the scope in which the nodes they process operate

	a `let` operator would apply to the parent scope, whenever it is

	node lists must carry that "scope" concept with them, so that each list
	knows what is their parent scope

	the scope is just an identifier copied along with the list; when an
	operator applies a change to the scope it basically publishes it
	and let the compiler handle modifying the given scope and applying the
	modified scope to all affected nodes

*/

#[derive(Debug, Eq, PartialEq)]
pub struct RawText(pub Input);

impl IsNode for RawText {}

pub struct ExpandRawText;

impl NodeOperator for ExpandRawText {
	fn evaluate(&self, context: &mut ResolveContext) {
		for (index, node) in context.nodes().clone().iter().enumerate() {
			if let Some(RawText(input)) = node.get::<RawText>() {
				let scanner = context.compiler().scanner();
				let mut cursor = input.start();
				let mut errors = Errors::new();
				let mut output = Vec::new();
				while let Some(node) = scanner.scan(&mut cursor, &mut errors) {
					output.push(node);
					if !errors.empty() {
						break;
					}
				}
				assert!(cursor.at_end() || !errors.empty());
				context.replace_index(index, output);
				if errors.len() > 0 {
					context.errors_mut().append(&errors);
				}
			}
		}
	}
}
