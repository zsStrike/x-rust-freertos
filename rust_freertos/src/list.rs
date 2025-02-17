use std::fmt;
use std::sync::{Arc, RwLock, Weak};

use crate::port::{portMAX_DELAY, TickType, UBaseType};
use crate::task_control::{TaskHandle, TCB};

impl fmt::Debug for ListItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ListItem with value: {}", self.item_value)
    }
}

/// * Descrpition:
///  Definition of the only type of object that a list can contain.
///
/// * Implemented by: Fan Jinhao
///
pub struct ListItem {
    /* The value being listed.  In most cases this is used to sort the list in descending order. */
    item_value: TickType,
    /* Pointer to the next ListItem_t in the list.
     * Previous design using WeakItemLink, this may 
     * result in using after dereferenced problem:
     * list.add(item1), list.add(item2) in build_list fn
     * call traverse(&list) in main will produce the bug.
     * the tests is in examples/test_list.rs
     */
    next: Option<ItemLink>,
    /* Pointer to the previous ListItem_t in the list. */
    prev: WeakItemLink,
    /* Pointer to the object (normally a TCB) that contains the list item.
     * There is therefore a two way link between the object containing the list item
     * and the list item itself. */
    owner: Weak<RwLock<TCB>>,
    /* Pointer to the list in which this list item is placed (if any). */
    container: Weak<RwLock<List>>,
}

pub type ItemLink = Arc<RwLock<ListItem>>;
pub type WeakItemLink = Weak<RwLock<ListItem>>;
pub type ListLink = Arc<RwLock<List>>;
pub type WeakListLink = Weak<RwLock<List>>;

impl Default for ListItem {
    fn default() -> Self {
        ListItem {
            /* The list end value is the highest possible value in the list to
            ensure it remains at the end of the list. */
            item_value: portMAX_DELAY,
            next: Default::default(),
            owner: Default::default(),
            prev: Default::default(),
            container: Default::default(),
        }
    }
}

impl ListItem {
    pub fn item_value(mut self, item_value: TickType) -> Self {
        self.item_value = item_value;
        self
    }

    pub fn owner(mut self, owner: TaskHandle) -> Self {
        self.owner = owner.into();
        self
    }

    pub fn set_container(&mut self, container: &Arc<RwLock<List>>) {
        self.container = Arc::downgrade(container);
    }

    fn remove(&mut self, link: WeakItemLink) -> UBaseType {
        /* The list item knows which list it is in.  Obtain the list from the list
        item. */
        let list = self
            .container
            .upgrade()
            .unwrap_or_else(|| panic!("Container not set"));
        let ret_val = list.write().unwrap().remove_item(&self, link);
        set_list_item_next(&self.prev, Arc::downgrade(self.next.as_ref().unwrap()));
        set_list_item_prev(&Arc::downgrade(self.next.as_ref().unwrap()), Weak::clone(&self.prev));
        self.container = Weak::new();
        ret_val
    }
}

/// * Descrpition:
///  Definition of the type of queue used by the scheduler.
///
/// * Implemented by: Fan Jinhao
///
#[derive(Clone)]
pub struct List {
    number_of_items: UBaseType,
    /* Used to walk through the list.
     * Points to the last item returned by a call to listGET_OWNER_OF_NEXT_ENTRY (). */
    index: WeakItemLink,
    /* List item that contains the maximum possible item value meaning
     * it is always at the end of the list and is therefore used as a marker. */
    list_end: ItemLink,
    list_start: ItemLink
}

impl Default for List {
    fn default() -> Self {
        /* The list structure contains a list item which is used to mark the
        end of the list.  To initialise the list the list end is inserted
        as the only list entry. */
        let list_end: ItemLink = Arc::new(RwLock::new(ListItem::default()));
        let list_start: ItemLink = Arc::new(RwLock::new(ListItem::default()));

        // /* The list end next and previous pointers point to itself so we know
        // when the list is empty. */
        list_start.write().unwrap().next = Some(Arc::clone(&list_end));
        list_end.write().unwrap().prev = Arc::downgrade(&list_start);
        
        List {
            index: Arc::downgrade(&list_end),
            list_end: list_end,
            list_start: list_start,
            number_of_items: 0,
        }
    }
}

fn set_list_item_next(item: &WeakItemLink, next: WeakItemLink) {
    let owned_item = item
        .upgrade()
        .unwrap_or_else(|| panic!("List item is None"));
    owned_item.write().unwrap().next = Some(next.upgrade().unwrap());
}

fn set_list_item_prev(item: &WeakItemLink, prev: WeakItemLink) {
    let owned_item = item
        .upgrade()
        .unwrap_or_else(|| panic!("List item is None"));
    owned_item.write().unwrap().prev = prev;
}

fn get_list_item_next(item: &WeakItemLink) -> WeakItemLink {
    let owned_item = item
        .upgrade()
        .unwrap_or_else(|| panic!("List item is None"));
    let next = Arc::downgrade((owned_item.read().unwrap().next.as_ref().unwrap()));
    next
}

fn get_list_item_prev(item: &WeakItemLink) -> WeakItemLink {        
    let owned_item = item
        .upgrade()
        .unwrap_or_else(|| panic!("List item is None"));
    let prev = Weak::clone(&(*owned_item.read().unwrap()).prev);
    prev
}

/// * Descrpition:
///  Access macro to retrieve the value of the list item.  The value can
///  represent anything - for example the priority of a task, or the time at
///  which a task should be unblocked.
///
/// * Implemented by: Fan Jinhao
///
/// # Arguments:
///
/// * Return:
///
pub fn get_list_item_value(item: &ItemLink) -> TickType {
    item.read().unwrap().item_value
}

/// * Descrpition:
///  Access macro to set the value of the list item.  In most cases the value is
///  used to sort the list in descending order.
///
/// * Implemented by: Fan Jinhao
///
/// # Arguments:
///
/// * Return:
///
pub fn set_list_item_value(item: &ItemLink, item_value: TickType) {
    item.write().unwrap().item_value = item_value;
}

fn get_weak_item_value(item: &WeakItemLink) -> TickType {
    let owned_item = item
        .upgrade()
        .unwrap_or_else(|| panic!("List item is None"));
    let value = owned_item.read().unwrap().item_value;
    value
}

fn set_weak_item_value(item: &WeakItemLink, item_value: TickType) {
    let owned_item = item
        .upgrade()
        .unwrap_or_else(|| panic!("List item is None"));
    owned_item.write().unwrap().item_value = item_value;
}

/// * Descrpition:
///  Return the list a list item is contained within (referenced from).
///
/// * Implemented by: Fan Jinhao
///
/// # Arguments:
///  `item` The list item being queried.
///
/// * Return:
///  A pointer to the List_t object that references the pxListItem
///
pub fn get_list_item_container(item: &ItemLink) -> Option<ListLink> {
    //let owned_item = item.upgrade().unwrap_or_else(|| panic!("List item is None"));
    let container = Weak::clone(&item.read().unwrap().container);
    container.upgrade()
}

/// * Descrpition:
///  Access macro to determine if a list contains any items.  The macro will
///  only have the value true if the list is empty.
///
/// * Implemented by: Fan Jinhao
///
/// # Arguments:
///
/// * Return:
///
pub fn list_is_empty(list: &ListLink) -> bool {
    list.read().unwrap().is_empty()
}

/// * Descrpition:
///  Access macro to return the number of items in the list.
///
/// * Implemented by: Fan Jinhao
///
/// # Arguments:
///
/// * Return:
///
pub fn current_list_length(list: &ListLink) -> UBaseType {
    list.read().unwrap().get_length()
}

/// * Descrpition:
///  Access function to get the owner of a list item.  The owner of a list item
///  is the object (usually a TCB) that contains the list item.
///
/// * Implemented by: Fan Jinhao
///
/// # Arguments:
///
/// * Return:
///
pub fn get_list_item_owner(item_link: &ItemLink) -> TaskHandle {
    let owner = Weak::clone(&item_link.read().unwrap().owner);
    owner.into()
}

/// * Descrpition:
///  Access function to set the owner of a list item.  The owner of a list item
///  is the object (usually a TCB) that contains the list item.
///
/// * Implemented by: Fan Jinhao
///
/// # Arguments:
///
/// * Return:
///
pub fn set_list_item_owner(item_link: &ItemLink, owner: TaskHandle) {
    item_link.write().unwrap().owner = owner.into()
}

/// * Descrpition:
///  Access function to obtain the owner of the next entry in a list.
///
///  The list member pxIndex is used to walk through a list.  Calling
///  listGET_OWNER_OF_NEXT_ENTRY increments pxIndex to the next item in the list
///  and returns that entry's pxOwner parameter.  Using multiple calls to this
///  function it is therefore possible to move through every item contained in
///  a list.
///
///  The pxOwner parameter of a list item is a pointer to the object that owns
///  the list item.  In the scheduler this is normally a task control block.
///  The pxOwner parameter effectively creates a two way link between the list
///  item and its owner.
///
/// * Implemented by: Fan Jinhao
///
/// # Arguments:
///  `list` The list from which the owner of the next item is to be
///  returned.
///
/// * Return:
/// The owner of next entry in list.
///
pub fn get_owner_of_next_entry(list: &ListLink) -> TaskHandle {
    let task = list.write().unwrap().get_owner_of_next_entry();
    task.into()
}

/// * Descrpition:
///  Access function to obtain the owner of the first entry in a list.  Lists
///  are normally sorted in ascending item value order.
///
///  This function returns the pxOwner member of the first item in the list.
///  The pxOwner parameter of a list item is a pointer to the object that owns
///  the list item.  In the scheduler this is normally a task control block.
///  The pxOwner parameter effectively creates a two way link between the list
///  item and its owner.
///
/// * Implemented by: Fan Jinhao
///
/// # Arguments:
///  `list` The list from which the owner of the head item is to be
///  returned.
///
/// * Return:
///
pub fn get_owner_of_head_entry(list: &ListLink) -> TaskHandle {
    let task = list.read().unwrap().get_owner_of_head_entry();
    task.into()
}

/// * Descrpition:
///  Check to see if a list item is within a list.  The list item maintains a
///  "container" pointer that points to the list it is in.  All this macro does
///  is check to see if the container and the list match.
///
/// * Implemented by: Fan Jinhao
///
/// # Arguments:
///
/// * Return:
///
pub fn is_contained_within(list: &ListLink, item_link: &ItemLink) -> bool {
    match get_list_item_container(&item_link) {
        Some(container) => Arc::ptr_eq(list, &container),
        None => false,
    }
}

/// * Descrpition:
///  Insert a list item into a list.  The item will be inserted into the list in
///  a position determined by its item value (descending item value order).
///
/// * Implemented by: Fan Jinhao
///
/// # Arguments:
///  `list` The list into which the item is to be inserted.
///
///  `item_link` The item that is to be placed in the list.
///
/// * Return:
///
pub fn list_insert(list: &ListLink, item_link: ItemLink) {
    /* Remember which list the item is in.  This allows fast removal of the
    item later. */
    item_link.write().unwrap().set_container(&list);
    println!("Set conatiner");
    list.write().unwrap().insert(Arc::downgrade(&item_link))
}

/// * Descrpition:
///  Insert a list item into a list.  The item will be inserted in a position
///  such that it will be the last item within the list returned by multiple
///  calls to listGET_OWNER_OF_NEXT_ENTRY.
///
///  The list member pxIndex is used to walk through a list.  Calling
///  listGET_OWNER_OF_NEXT_ENTRY increments pxIndex to the next item in the list.
///  Placing an item in a list using vListInsertEnd effectively places the item
///  in the list position pointed to by pxIndex.  This means that every other
///  item within the list will be returned by listGET_OWNER_OF_NEXT_ENTRY before
///  the pxIndex parameter again points to the item being inserted.
///
/// * Implemented by: Fan Jinhao
///
/// # Arguments:
///  `list` The list into which the item is to be inserted.
///
///  `item_link` The list item to be inserted into the list.
///
/// * Return:
///
pub fn list_insert_end(list: &ListLink, item_link: ItemLink) {
    /* Insert a new list item into pxList, but rather than sort the list,
    makes the new list item the last item to be removed by a call to
    listGET_OWNER_OF_NEXT_ENTRY(). */

    /* Remember which list the item is in. */
    item_link.write().unwrap().set_container(&list);

    list.write().unwrap().insert_end(Arc::downgrade(&item_link))
}

/// * Descrpition:
///  Remove an item from a list.  The list item has a pointer to the list that
///  it is in, so only the list item need be passed into the function.
///
/// * Implemented by: Fan Jinhao
///
/// # Arguments:
///  `item_link` The item to be removed.  The item will remove itself from
///  the list pointed to by it's pxContainer parameter.
///
/// * Return:
///  The number of items that remain in the list after the list item has
///  been removed.
///
pub fn list_remove(item_link: ItemLink) -> UBaseType {
    item_link
        .write()
        .unwrap()
        .remove(Arc::downgrade(&item_link))
}

pub fn traverse(list: &ListLink) {
    let mut iter = Weak::clone(&list.read().unwrap().index);
    list.write().unwrap().increment_index();
    while !Weak::ptr_eq(&iter, &list.read().unwrap().index) {
        if !Weak::ptr_eq(&Arc::downgrade(&list.read().unwrap().list_end), &list.read().unwrap().index) {
            println!("item value = {}, ref_count = {}", 
                get_weak_item_value(&list.read().unwrap().index),
                Arc::strong_count(&list.read().unwrap().index.upgrade().unwrap()));
        }
        list.write().unwrap().increment_index();
    }
}

impl List {
    fn insert(&mut self, item_link: WeakItemLink) {
        println!("in");
        let value_of_insertion = get_weak_item_value(&item_link);
        /* Insert the new list item into the list, sorted in xItemValue order.

        If the list already contains a list item with the same item value then the
        new list item should be placed after it.  This ensures that TCB's which are
        stored in ready lists (all of which have the same xItemValue value) get a
        share of the CPU.  However, if the xItemValue is the same as the back marker
        the iteration loop below will not end.  Therefore the value is checked
        first, and the algorithm slightly modified if necessary. */
        let item_to_insert = if value_of_insertion == portMAX_DELAY {
            get_list_item_prev(&Arc::downgrade(&self.list_end))
        } else {
            /* *** NOTE ***********************************************************
              If you find your application is crashing here then likely causes are
              listed below.  In addition see http://www.freertos.org/FAQHelp.html for
              more tips, and ensure configASSERT() is defined!
              http://www.freertos.org/a00110.html#configASSERT

              1) Stack overflow -
              see http://www.freertos.org/Stacks-and-stack-overflow-checking.html
              2) Incorrect interrupt priority assignment, especially on Cortex-M
              parts where numerically high priority values denote low actual
              interrupt priorities, which can seem counter intuitive.  See
              http://www.freertos.org/RTOS-Cortex-M3-M4.html and the definition
              of configMAX_SYSCALL_INTERRUPT_PRIORITY on
              http://www.freertos.org/a00110.html
              3) Calling an API function from within a critical section or when
              the scheduler is suspended, or calling an API function that does
              not end in "FromISR" from an interrupt.
              4) Using a queue or semaphore before it has been initialised or
              before the scheduler has been started (are interrupts firing
              before vTaskStartScheduler() has been called?).
            **********************************************************************/
            // let mut iterator = Arc::downgrade(&self.list_end);
            // loop {
            //     /* There is nothing to do here, just iterating to the wanted
            //     insertion position. */
            //     let next = get_list_item_next(&iterator);
            //     if get_weak_item_value(&next) > value_of_insertion {
            //         break iterator;
            //     }
            //     iterator = next;
            // }
            let mut iterator = Arc::downgrade(&self.list_start);
            loop {
                let next = get_list_item_next(&iterator);
                if get_weak_item_value(&next) > value_of_insertion {
                    break iterator;
                }
                iterator = next;
            }
        };

        let prev = Weak::clone(&item_to_insert);
        let next = get_list_item_next(&item_to_insert);

        set_list_item_next(&item_link, Weak::clone(&next));
        set_list_item_prev(&item_link, Weak::clone(&prev));
        set_list_item_next(&prev, Weak::clone(&item_link));
        set_list_item_prev(&next, Weak::clone(&item_link));

        self.number_of_items += 1;
    }

    fn insert_end(&mut self, item_link: WeakItemLink) {
        let prev = get_list_item_prev(&self.index);
        let next = Weak::clone(&self.index);
        set_list_item_next(&item_link, Weak::clone(&next));
        set_list_item_prev(&item_link, Weak::clone(&prev));
        set_list_item_next(&prev, Weak::clone(&item_link));
        set_list_item_prev(&next, Weak::clone(&item_link));

        self.number_of_items += 1;
    }

    fn remove_item(&mut self, item: &ListItem, link: WeakItemLink) -> UBaseType {
        // TODO: Find a more effiecient
        if Weak::ptr_eq(&link, &self.index) {
            self.index = Weak::clone(&item.prev);
            if Weak::ptr_eq(&Arc::downgrade(&self.list_start), &self.index) {
                self.index = Weak::clone(&Arc::downgrade(&self.list_end));
            }
        }

        self.number_of_items -= 1;

        self.number_of_items
    }

    fn is_empty(&self) -> bool {
        self.number_of_items == 0
    }

    fn get_length(&self) -> UBaseType {
        self.number_of_items
    }

    fn increment_index(&mut self) {
        if Weak::ptr_eq(&self.index, &Arc::downgrade(&self.list_end)) {
            self.index = get_list_item_next(&Arc::downgrade(&self.list_start));
        } else {
            self.index = get_list_item_next(&self.index);
        }
    }

    // fn decrease_index(&mut self) {
    //     if Weak::ptr_eq(&self.index, &Arc::downgrade(&self.list_start)) {
    //         self.index = get_list_item_prev(&Arc::downgrade(&self.list_end));
    //     } else {
    //         self.index = get_list_item_prev(&self.index);
    //         if Weak::ptr_eq(&self.index, &Arc::downgrade(&self.list_start)) {
    //             self.index = get_list_item_prev(&Arc::downgrade(&self.list_end));
    //         }            
    //     }
    // }

    fn get_owner_of_next_entry(&mut self) -> Weak<RwLock<TCB>> {
        self.increment_index();
        if Weak::ptr_eq(&self.index, &Arc::downgrade(&self.list_end)) {
            self.index = get_list_item_next(&Arc::downgrade(&self.list_start));
        }
        let owned_index = self
            .index
            .upgrade()
            .unwrap_or_else(|| panic!("List item is None"));
        let owner = Weak::clone(&owned_index.read().unwrap().owner);
        owner
    }

    fn get_owner_of_head_entry(&self) -> Weak<RwLock<TCB>> {
        let list_end = get_list_item_next(&Arc::downgrade(&self.list_start));
        let owned_index = list_end
            .upgrade()
            .unwrap_or_else(|| panic!("List item is None"));
        let owner = Weak::clone(&owned_index.read().unwrap().owner);
        owner
    }
}

