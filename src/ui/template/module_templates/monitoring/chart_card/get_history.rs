use crate::module::Module;

use super::ChartCard;

impl ChartCard {
    pub(super) fn get_history(module: &Module) -> Vec<(f64, f64, i64)> {
        if let Some(history_json) = module.config.bindings.get("_chart_history") {
            if let Some(arr) = history_json.as_array() {
                return arr.iter()
                    .filter_map(|v| {
                        let obj = v.as_object()?;
                        let a = obj.get("a")?.as_f64()?;
                        let b = obj.get("b")?.as_f64()?;
                        let c = obj.get("c")?.as_i64()?;
                        Some((a, b, c))
                    })
                    .collect();
            }
        }
        Vec::new()
    }
}
