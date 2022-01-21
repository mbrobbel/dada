use std::path::Path;

use dada_execute::kernel::BufferKernel;
use dada_ir::filename::Filename;
use dada_ir::span::LineColumn;
use salsa::DebugWithDb;

use crate::test_harness::QueryKind;

use super::{Errors, Query};

impl super::Options {
    pub(super) async fn perform_heap_graph_query_on_db(
        &self,
        db: &dada_db::Db,
        path: &Path,
        query_index: usize,
        filename: Filename,
        query: &Query,
        errors: &mut Errors,
    ) -> eyre::Result<()> {
        assert!(matches!(query.kind, QueryKind::HeapGraph));

        let breakpoint = dada_breakpoint::breakpoint::find(
            db,
            filename,
            LineColumn::new1(query.line, query.column),
        );

        let mut kernel = BufferKernel::new().breakpoint(breakpoint);

        if let Some(breakpoint) = breakpoint {
            kernel.append(&format!(
                "# Breakpoint: {:?}\n",
                breakpoint.span(db).debug(db),
            ));
        }

        match db.function_named(filename, "main") {
            Some(function) => {
                kernel.interpret_and_buffer(db, function, vec![]).await;
            }
            None => {
                kernel.append(&format!(
                    "no `main` function in `{}`\n",
                    filename.as_str(db)
                ));
            }
        }

        let actual_output = kernel.take_buffer();

        let output_matched = query.message.is_match(&actual_output);

        let ref_path = path.join(format!("HeapGraph-{query_index}.ref"));
        self.check_output_against_ref_file(actual_output, &ref_path, errors)?;

        if !output_matched {
            eyre::bail!("query regex `{:?}` did not match the output", query.message);
        }

        Ok(())
    }
}
