// src/ui/template.rs
use crate::module::Module;
use ratatui::prelude::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::any::Any;

pub mod module_templates;

/// Every UI widget implements this
pub trait UiTemplate: Any + Send + Sync + Debug {
    fn render_overview_cta(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module);
    fn render_detail(&self, area: Rect, buf: &mut Buffer, module: &mut Module);
    fn required_bindings(&self) -> &'static [&'static str];
    fn docs(&self) -> &'static str;
}

/// Factory type
pub type TemplateFactory = fn() -> Box<dyn UiTemplate>;

/// GLOBAL REGISTRY - Register all your templates here
lazy_static::lazy_static! {
    pub static ref TEMPLATE_REGISTRY: HashMap<&'static str, TemplateFactory> = {
        let mut map = HashMap::new();

        // Monitoring templates
        map.insert("gauge_card", gauge_card_factory as TemplateFactory);
        map.insert("history_chart", history_chart_factory as TemplateFactory);
        map.insert("status_badge_card", status_badge_factory as TemplateFactory);
        map.insert("chart_card", chart_card_factory as TemplateFactory);

        // Control templates
        map.insert("toggle_switch", toggle_switch_factory as TemplateFactory);

        // Com templates
        map.insert("activity_card", activity_card_factory as TemplateFactory);
        map.insert("llm_card", llm_card_factory as TemplateFactory);

        // System templates
        map.insert("overseer_card", overseer_card_factory as TemplateFactory);

        // Planning templates
        map.insert("side_quest_card", side_quest_card_factory as TemplateFactory);

        map
    };
}

// Factory functions
fn gauge_card_factory() -> Box<dyn UiTemplate> {
    Box::new(module_templates::monitoring::gauge_card::GaugeCard::default())
}

fn chart_card_factory() -> Box<dyn UiTemplate> {
    Box::new(module_templates::monitoring::chart_card::ChartCard::default())
}

fn history_chart_factory() -> Box<dyn UiTemplate> {
    Box::new(module_templates::monitoring::history_chart_card::HistoryChart)
}

fn activity_card_factory() -> Box<dyn UiTemplate> {
    Box::new(module_templates::com::activity_card::ActivityCard)
}

fn toggle_switch_factory() -> Box<dyn UiTemplate> {
    Box::new(module_templates::control::toggle_switch_card::ToggleSwitch)
}

fn status_badge_factory() -> Box<dyn UiTemplate> {
    Box::new(module_templates::monitoring::status_badge_card::StatusBadge)
}

fn llm_card_factory() -> Box<dyn UiTemplate> {
    Box::new(module_templates::knowledge::llm_card::LlmCard)
}

fn overseer_card_factory() -> Box<dyn UiTemplate> {
    Box::new(module_templates::system::overseer_card::OverseerCard)
}

fn side_quest_card_factory() -> Box<dyn UiTemplate> {
    Box::new(module_templates::planning::side_quest_card::SideQuestCard)
}

/// Helper to get a template instance
pub fn get_template(name: &str) -> Option<Box<dyn UiTemplate>> {
    TEMPLATE_REGISTRY.get(name).map(|&factory| factory())
}
