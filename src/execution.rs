use crate::stepper_outcome::{StepperOutcomeInner};
use crate::instructions::Instruction;

pub fn exec(init_user_id: String, aqua: String, data: String) -> StepperOutcomeInner {
    log::info!(
        "stepper invoked with user_id = {}, aqua = {:?}, data = {:?}",
        init_user_id,
        aqua,
        data
    );

    let outcome = StepperOutcomeInner {
        data,
        next_peer_pks: vec![init_user_id],
    };

    let parsed_aqua = match serde_sexpr::from_str::<Vec<Instruction>>(&aqua) {
        Ok(parsed) => parsed,
        Err(e) => {
            log::error!("supplied aqua script can't be parsed: {:?}", e);

            return outcome;
        }
    };
    log::info!("parsed_aqua: {:?}", parsed_aqua);

    crate::stepper::execute(parsed_aqua);

    outcome
}