fn bmh_ground_item_blast(ignored: bool, item: &ItemEntity) -> bool {
    ignored || horizon_client_lib::bmh_ground_items::protected(&item.stack.item)
}
