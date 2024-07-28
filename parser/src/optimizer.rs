struct Write<'a>{
    prefix: &'a str,
    pattern: &'a str,
    args: &'a str,
    postfix: &'a str
}

fn find_closing(src: &str) -> usize{
    let mut escaped = false;
    for (i, c) in src.char_indices(){
        match c{
            '\\' => escaped = !escaped,
            '"' if !escaped => return i,
            _ => {
                escaped = false;
                ()
            }
        }
    }
    panic!("unmatched quote near {}", src);
}

static OPEN: &str = "write!(f, \"";
static CLOSE: &str = ")?;";

impl<'a> Write<'a>{
    fn first(src: &'a str) -> Option<Self>{
        let offset = match src.find(OPEN){
            Some(offset) => offset,
            None => return None
        };
        let prefix = &src[..offset];
        let postfix = &src[offset + OPEN.len()..];
        let offset = find_closing(postfix);
        let pattern = &postfix[..offset];
        let postfix = &postfix[offset + 1..];
        let offset = postfix.find(CLOSE).unwrap();

        Some(Self{
            prefix,
            pattern,
            args: &postfix[..offset],
            postfix: &postfix[offset + CLOSE.len()..]
        })
    }

    fn next(&self) -> Option<Self>{
        Self::first(self.postfix)
    }
}

fn close(format: &str, args: &str, out: &mut String){
    if !format.is_empty(){
        out.push_str(OPEN);
        out.push_str(&format);
        out.push('"');
        out.push_str(&args);
        out.push_str(CLOSE);
    }
}

pub(crate) fn optimize(src: String) -> String{
    let mut first = match Write::first(&src){
        Some(first) => first,
        None => return src
    };
    let mut out = String::with_capacity(src.len());
    let mut format = String::new();
    let mut args = String::new();
    loop{
        match first.next(){
            Some(next) => {
                if first.prefix.is_empty(){
                    format.push_str(first.pattern);
                    args.push_str(first.args);
                }
                else{
                    close(&format, &args, &mut out);
                    out.push_str(first.prefix);
                    format.clear();
                    args.clear();
                    format.push_str(first.pattern);
                    args.push_str(first.args);
                }
                first = next;
            },
            None => {
                if first.prefix.is_empty(){
                    out.push_str(OPEN);
                    out.push_str(&format);
                    out.push_str(first.pattern);
                    out.push('"');
                    out.push_str(&args);
                    out.push_str(first.args);
                    out.push_str(CLOSE);
                }
                else{
                    close(&format, &args, &mut out);
                    out.push_str(first.prefix);
                    out.push_str(OPEN);
                    out.push_str(first.pattern);
                    out.push('"');
                    out.push_str(first.args);
                    out.push_str(CLOSE);
                }
                out.push_str(first.postfix);
                return out;
            }
        }
    }
}

#[cfg(test)]
mod tests{
    use super::optimize;

    #[test]
    fn test_optimize(){
        assert_eq!(
            optimize("if self.some.as_bool(){write!(f, \"Hello\")?;}else{write!(f, \"World\")?;}".to_string()),
            "if self.some.as_bool(){write!(f, \"Hello\")?;}else{write!(f, \"World\")?;}"
        );
    }
}