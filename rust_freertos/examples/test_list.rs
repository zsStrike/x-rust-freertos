extern crate rust_freertos;

use rust_freertos::{*, list::*};
use std::sync::{Arc, RwLock, Weak};

fn build_list() -> Arc<RwLock<List>> {
    let lst = Arc::new(RwLock::new(List::default()));
    let item1 = Arc::new(RwLock::new(ListItem::default()));
    set_list_item_value(&item1, 100);
    list_insert_end(&lst, Arc::clone(&item1));
    let item2 = Arc::new(RwLock::new(ListItem::default()));
    set_list_item_value(&item2, 200);
    list_insert_end(&lst, Arc::clone(&item2));
    return lst;
}

fn test_insert(list: &Arc<RwLock<List>>) {
    println!("====================test_insert=======================");
    let item1 = Arc::new(RwLock::new(ListItem::default()));
    set_list_item_value(&item1, 36);
    list_insert(&list, Arc::clone(&item1));
    let item2 = Arc::new(RwLock::new(ListItem::default()));
    set_list_item_value(&item2, 178);
    list_insert(&list, Arc::clone(&item2));
    let item3 = Arc::new(RwLock::new(ListItem::default()));
    set_list_item_value(&item3, 324);
    list_insert(&list, Arc::clone(&item3));  
    let item4 = Arc::new(RwLock::new(ListItem::default()));
    set_list_item_value(&item4, 145);
    list_insert(&list, Arc::clone(&item4));          
}

fn test_remove(list: &Arc<RwLock<List>>) {
    println!("====================test_remove=======================");
    let item1 = Arc::new(RwLock::new(ListItem::default()));
    set_list_item_value(&item1, 9);
    list_insert(&list, Arc::clone(&item1));
    traverse(&list); 
    let item2 = Arc::new(RwLock::new(ListItem::default()));
    set_list_item_value(&item2, 199);
    list_insert(&list, Arc::clone(&item2));
    traverse(&list); 
    let item3 = Arc::new(RwLock::new(ListItem::default()));
    set_list_item_value(&item3, 876);
    list_insert(&list, Arc::clone(&item3));  
    traverse(&list); 

    println!("remove item = {}", get_list_item_value(&item1));
    list_remove(item1);
    traverse(&list);   
    println!("remove item = {}", get_list_item_value(&item2));
    list_remove(item2);
    traverse(&list);  
    println!("remove item = {}", get_list_item_value(&item3));
    list_remove(item3);
    traverse(&list);               
}


fn main() {
    let lst = build_list();
    println!("lst size = {}", current_list_length(&lst));
    traverse(&lst); 
    test_insert(&lst);
    println!("lst size = {}", current_list_length(&lst));
    traverse(&lst); 
    test_remove(&lst);
}
