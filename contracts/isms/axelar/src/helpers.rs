use cosmwasm_std::Event;

pub fn new_event(name: &str) -> Event {
    Event::new(format!("hpl_ism_axelar::{}", name))
}
