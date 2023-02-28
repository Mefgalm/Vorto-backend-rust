use std::{any::Any, fmt::Display};

pub fn to_sql_value<T: Any + Display>(value: &T) -> String {
    let value_any = value as &dyn Any;

    if value_any.is::<String>() || value_any.is::<&str>() {
        //SQL injection protect        
        format!("'{}'", value.to_string().replace("'", "''"))
    } else {
        value.to_string()
    }
}

pub fn in_qry<T: Any + Display>(field: &str, values: &Vec<T>) -> String {
    format!(
        "{} IN ({})",
        field,
        values.into_iter().map(to_sql_value).collect::<Vec<_>>().join(",")
    )
}

macro_rules! run_qry {
    ($qry: expr, $fn_name: ident, $pool: expr, $tx: expr) => {
        if let Some(t) = $tx {
            $qry.$fn_name(t).await?
        } else {
            $qry.$fn_name($pool).await?
        }
    };
}
