//! 辅助函数：DropDown + StringList（替代已弃用的 ComboBoxText）

use gtk4::prelude::*;
use gtk4::{DropDown, StringList};

/// 用字符串列表填充 StringList 并设置 DropDown 选中第一项。
pub fn set_drop_down_model_and_first<I, S>(drop_down: &DropDown, string_list: &StringList, items: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let v: Vec<String> = items.into_iter().map(|s| s.as_ref().to_string()).collect();
    let refs: Vec<&str> = v.iter().map(|s| s.as_str()).collect();
    let n = string_list.n_items();
    string_list.splice(0, n, &refs);
    drop_down.set_selected(0);
}

/// 获取当前选中项文本（模型为 StringList 时）。
pub fn drop_down_selected_text(drop_down: &DropDown) -> Option<String> {
    drop_down
        .selected_item()
        .and_then(|o| o.downcast::<gtk4::StringObject>().ok())
        .map(|s| s.string().to_string())
}
