use crate::{parser::Parser, token_test::Identifier};

use dada_ir::{
    class::Class, code::Code, effect::Effect, function::Function, item::Item, kw::Keyword,
};

use super::OrReportError;

impl<'db> Parser<'db> {
    pub(crate) fn parse_items(&mut self) -> Vec<Item> {
        let mut items = vec![];
        while self.tokens.peek().is_some() {
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else {
                let span = self.tokens.last_span();
                self.tokens.consume();
                dada_ir::error!(span.in_file(self.filename), "unexpected token").emit(self.db);
            }
        }
        items
    }

    fn parse_item(&mut self) -> Option<Item> {
        if let Some(class) = self.parse_class() {
            Some(Item::Class(class))
        } else {
            self.parse_function().map(Item::Function)
        }
    }

    fn parse_class(&mut self) -> Option<Class> {
        let (class_span, _) = self.eat(Keyword::Class)?;
        let (class_name_span, class_name) = self
            .eat(Identifier)
            .or_report_error(self, || "expected a class name")?;
        let (_, field_tokens) = self
            .delimited('(')
            .or_report_error(self, || "expected class parameters")?;
        Some(Class::new(
            self.db,
            class_name,
            field_tokens,
            self.span_consumed_since(class_span).in_file(self.filename),
            class_name_span.in_file(self.filename),
        ))
    }

    fn parse_function(&mut self) -> Option<Function> {
        let (effect_span, effect) = if let Some((span, _)) = self.eat(Keyword::Async) {
            (Some(span), Effect::Async)
        } else {
            (None, Effect::Default)
        };
        let (fn_span, _) = self
            .eat(Keyword::Fn)
            .or_report_error(self, || "expected `fn`".to_string())?;
        let (func_name_span, func_name) = self
            .eat(Identifier)
            .or_report_error(self, || "expected function name".to_string())?;
        let (_, parameter_tokens) = self
            .delimited('(')
            .or_report_error(self, || "expected function parameters".to_string())?;
        let (_, body_tokens) = self
            .delimited('{')
            .or_report_error(self, || "expected function body".to_string())?;
        let code = Code::new(effect, Some(parameter_tokens), body_tokens);
        let start_span = effect_span.unwrap_or(fn_span);
        Some(Function::new(
            self.db,
            func_name,
            code,
            self.span_consumed_since(start_span).in_file(self.filename),
            func_name_span.in_file(self.filename),
            effect_span.unwrap_or(fn_span).in_file(self.filename),
        ))
    }
}
