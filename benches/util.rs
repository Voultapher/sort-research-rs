pub fn pin_thread_to_core() {
    use std::cell::Cell;
    let pin_core_id: usize = 2;

    thread_local! {static AFFINITY_ALREADY_SET: Cell<bool> = Cell::new(false); }

    // Set affinity only once per thread.
    if !AFFINITY_ALREADY_SET.get() {
        if let Some(core_id_2) = core_affinity::get_core_ids()
            .as_ref()
            .and_then(|ids| ids.get(pin_core_id))
        {
            core_affinity::set_for_current(*core_id_2);
        }

        AFFINITY_ALREADY_SET.set(true);
    }
}
