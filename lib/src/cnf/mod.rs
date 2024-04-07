// Specifies how many concurrent jobs can be buffered in the worker channel.
pub const MAX_CONCURRENT_TASKS: usize = 64;

// Specifies how many subqueries will be processed recursively before the query fails.
pub const MAX_RECURSIVE_QUERIES: usize = 16;

// The characters which are supported in server record IDs.
pub const ID_CHARS: [char; 36] = [
	'0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
	'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];
