use crate::ctx::Context;
use crate::dbs::Transaction;
use crate::doc::CursorDoc;
use crate::err::Error;
use crate::idx::planner::executor::QueryExecutor;
use crate::sql::{Thing, Value};

fn get_execution_context<'a>(
	ctx: &'a Context<'_>,
	txn: Option<&'a Transaction>,
	doc: Option<&'a CursorDoc<'_>>,
) -> Option<(&'a Transaction, &'a QueryExecutor, &'a CursorDoc<'a>, &'a Thing)> {
	if let Some(txn) = txn {
		if let Some(doc) = doc {
			if let Some(thg) = doc.rid {
				if let Some(pla) = ctx.get_query_planner() {
					if let Some(exe) = pla.get_query_executor(&thg.tb) {
						return Some((txn, exe, doc, thg));
					}
				}
			}
		}
	}
	None
}

pub async fn score(
	(ctx, txn, doc): (&Context<'_>, Option<&Transaction>, Option<&CursorDoc<'_>>),
	(match_ref,): (Value,),
) -> Result<Value, Error> {
	if let Some((txn, exe, doc, thg)) = get_execution_context(ctx, txn, doc) {
		exe.score(txn, &match_ref, thg, doc.doc_id).await
	} else {
		Ok(Value::None)
	}
}

pub async fn highlight(
	(ctx, txn, doc): (&Context<'_>, Option<&Transaction>, Option<&CursorDoc<'_>>),
	(prefix, suffix, match_ref): (Value, Value, Value),
) -> Result<Value, Error> {
	if let Some((txn, exe, doc, thg)) = get_execution_context(ctx, txn, doc) {
		exe.highlight(txn, thg, prefix, suffix, &match_ref, doc.doc.as_ref()).await
	} else {
		Ok(Value::None)
	}
}

pub async fn offsets(
	(ctx, txn, doc): (&Context<'_>, Option<&Transaction>, Option<&CursorDoc<'_>>),
	(match_ref,): (Value,),
) -> Result<Value, Error> {
	if let Some((txn, exe, _, thg)) = get_execution_context(ctx, txn, doc) {
		exe.offsets(txn, thg, &match_ref).await
	} else {
		Ok(Value::None)
	}
}
