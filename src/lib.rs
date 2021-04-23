#![feature(option_result_unwrap_unchecked)]

use core::cell::UnsafeCell;

#[derive(Debug)]
pub struct Link {
    next: UnsafeCell<*const Link>,
}

pub struct List {
    last: Option<Link>,
}

impl Link {
    const unsafe fn new() -> Link {
        Link {
            next: UnsafeCell::new(core::ptr::null()),
        }
    }

    const unsafe fn new_linked(link: *const Link) -> Link {
        Link {
            next: UnsafeCell::new(link),
        }
    }

    unsafe fn link(&self, next: &Link) {
        *self.next.get() = next as *const Link;
    }

    unsafe fn next_ptr(&self) -> *const Link {
        *self.next.get()
    }

    unsafe fn next(&self) -> &Link {
        &*self.next_ptr()
    }
}

// public
impl List {
    /// creates a new, empty list
    pub const fn new() -> Self {
        List { last: None }
    }

    /// returns true if list does not contain any elements
    pub fn is_empty(&self) -> bool {
        self.last.is_none()
    }

    /// Inserts element at the beginning of the list
    /// Complexity: O(1)
    pub fn lpush(&mut self, element: &mut Link) {
        if self.is_empty() {
            unsafe { self.push_initial_element(element) };
        } else {
            unsafe {
                element.link(self.head());
                self.tail().link(element);
            };
        }
    }

    /// Remove and return element from the beginning of the list
    /// Complexity: O(1)
    pub fn lpop(&mut self) -> Option<&Link> {
        if self.is_empty() {
            None
        } else {
            unsafe {
                let head = self.head_ptr();
                if self.tail_ptr() == head {
                    self.last = None;
                } else {
                    self.tail().link(self.head().next());
                }

                Some(&*head)
            }
        }
    }

    /// Returns the first element in the list without removing it
    /// Complexity: O(1)
    pub fn lpeek(&self) -> Option<&Link> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.head() })
        }
    }

    /// Inserts element at the end of the list
    /// Complexity: O(1)
    pub fn rpush(&mut self, element: &mut Link) {
        self.lpush(element);
        self.last = Some(unsafe { Link::new_linked(element) });
    }

    /// Remove and return element from the end of the list
    /// Complexity: O(1)
    pub fn rpop(&mut self) -> Option<&Link> {
        if self.is_empty() {
            None
        } else {
            self.remove(unsafe { self.tail() })
        }
    }

    /// Returns the last element in the list without removing it
    /// Complexity: O(1)
    pub fn rpeek(&self) -> Option<&Link> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.tail() })
        }
    }

    /// Rotates list (first becomes last, second becomes first)
    /// Complexity: O(1)
    pub fn lpoprpush(&mut self) {
        if !self.is_empty() {
            unsafe { self.last().link(self.head()) };
        }
    }

    /// Find element
    /// Complexity: O(n)
    pub fn find(&self, element: &Link) -> Option<&Link> {
        unsafe { self.find_previous(element).and_then(|x| Some(x.next())) }
    }

    /// Remove and return element
    /// Complexity: O(n)
    pub fn remove(&self, element: &Link) -> Option<&Link> {
        if self.is_empty() {
            None
        } else if unsafe { self.head_ptr() } == element as *const _ {
            // this deals with the case of removing the only element,
            // at the cost of comparing head to element twice
            self.lpop()
        } else {
            unsafe {
                // storing element here so we can return it from the closure
                let res = element as *const _;
                self.find_previous(element).and_then(|x| {
                    x.link(x.next().next());
                    Some(&*res)
                })
            }
        }
    }
}

impl core::fmt::Debug for List {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if self.is_empty() {
            write!(f, "List {{}}")
        } else {
            unsafe {
                write!(
                    f,
                    "List {{ {:x} {:x}:{:x} {:x}:{:x}",
                    self.last().next_ptr() as usize,
                    self.tail() as *const _ as usize,
                    self.tail().next_ptr() as usize,
                    self.head() as *const _ as usize,
                    self.head().next_ptr() as usize,
                )
            }
        }
    }
}

/// internal
impl List {
    unsafe fn last(&self) -> &Link {
        &self.last.as_ref().unwrap_unchecked()
    }

    unsafe fn tail(&self) -> &Link {
        self.last().next()
    }

    unsafe fn tail_ptr(&self) -> *const Link {
        self.last().next_ptr()
    }

    unsafe fn head(&self) -> &Link {
        self.tail().next()
    }

    unsafe fn head_ptr(&self) -> *const Link {
        self.tail().next()
    }

    unsafe fn push_initial_element(&mut self, element: &mut Link) {
        element.link(element);
        self.last = Some(Link::new_linked(element));
    }

    unsafe fn find_previous(&self, element: &Link) -> Option<&Link> {
        if self.is_empty() {
            return None;
        }
        let mut pos = self.tail();
        let tail_ptr = pos as *const Link;
        let element_ptr = element as *const Link;
        loop {
            let next_ptr = pos.next_ptr();
            if next_ptr == element_ptr {
                return Some(pos);
            }
            if next_ptr == tail_ptr {
                return None;
            }
            pos = pos.next();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lpush_lpop_1() {
        let mut list = List::new();
        assert!(list.lpop().is_none());

        let mut node = unsafe { Link::new() };

        list.lpush(&mut node);

        assert!(unsafe { node.next_ptr() } == &node as *const Link);
        assert!(list.lpop().unwrap() as *const Link == &node as *const Link);
        assert!(list.lpop().is_none());
    }

    #[test]
    fn test_lpush_lpop_2() {
        let mut list = List::new();
        assert!(list.lpop().is_none());

        let mut node = unsafe { Link::new() };
        list.lpush(&mut node);
        assert!(unsafe { node.next_ptr() } == &node as *const Link);

        let mut node2 = unsafe { Link::new() };
        list.lpush(&mut node2);

        assert!(unsafe { node2.next_ptr() } == &node as *const Link);
        assert!(unsafe { node.next_ptr() } == &node2 as *const Link);
        assert!(unsafe { list.last().next_ptr() == &node as *const Link });

        assert!(list.lpop().unwrap() as *const Link == &node2 as *const Link);
        assert!(list.lpop().unwrap() as *const Link == &node as *const Link);
        assert!(list.lpop().is_none());
    }

    #[test]
    fn test_lpush_lpop_3() {
        let mut list = List::new();
        assert!(list.lpop().is_none());

        let mut node = unsafe { Link::new() };
        list.lpush(&mut node);
        assert!(unsafe { node.next_ptr() } == &node as *const Link);

        let mut node2 = unsafe { Link::new() };
        list.lpush(&mut node2);

        let mut node3 = unsafe { Link::new() };
        list.lpush(&mut node3);

        assert!(unsafe { node.next_ptr() } == &node3 as *const Link);
        assert!(unsafe { node2.next_ptr() } == &node as *const Link);
        assert!(unsafe { node3.next_ptr() } == &node2 as *const Link);
        assert!(unsafe { list.tail_ptr() == &node as *const Link });

        assert!(list.lpop().unwrap() as *const Link == &node3 as *const Link);
        assert!(unsafe { node.next_ptr() } == &node2 as *const Link);
        assert!(unsafe { node2.next_ptr() } == &node as *const Link);
        assert!(unsafe { list.tail_ptr() == &node as *const Link });
        //assert!(unsafe { node3.next_ptr() } == core::ptr::null());

        assert!(list.lpop().unwrap() as *const Link == &node2 as *const Link);
        assert!(unsafe { node.next_ptr() } == &node as *const Link);
        assert!(unsafe { list.tail_ptr() == &node as *const Link });
        //assert!(unsafe { node2.next_ptr() } == core::ptr::null());

        assert!(list.lpop().unwrap() as *const Link == &node as *const Link);
        //assert!(unsafe { node.next_ptr() } == core::ptr::null());
        assert!(list.last.is_none());

        assert!(list.lpop().is_none());
    }

    #[test]
    fn test_lpoprpush() {
        let mut list = List::new();

        let mut node = unsafe { Link::new() };
        let mut node2 = unsafe { Link::new() };

        list.lpush(&mut node);
        list.lpush(&mut node2);
        list.lpoprpush();

        assert!(list.lpop().unwrap() as *const _ == &node as *const _);
        assert!(list.lpop().unwrap() as *const _ == &node2 as *const _);
        assert!(list.lpop().is_none());
    }
}
