use super::*;
#[test]
fn input_check() {
    assert_eq!(wants_input("foo , bar"), true);
    assert_eq!(wants_input("foo . bar"), false);
}

#[cfg(test)]
mod transpiler {
    use crate::Exec;
    #[test]
    fn in_out() {
        assert_eq!(
            Exec::prog(",.")
                .input(Some(String::from("a")))
                .transpile()
                .unwrap(),
            String::from("a")
        );
    }

    #[test]
    fn loop_math() {
        assert_eq!(
            Exec::prog("+++++[>++++++++++<-]>-.").transpile().unwrap(),
            String::from("1")
        );
    }

    #[test]
    #[should_panic]
    fn out_of_bounds() {
        Exec::prog("<+").transpile().unwrap();
    }

    #[test]
    #[should_panic]
    fn out_of_input() {
        Exec::prog(",").transpile().unwrap();
    }
}

#[cfg(test)]
mod interpreter {
    use crate::Exec;
    #[test]
    fn in_out() {
        assert_eq!(
            Exec::prog(",.")
                .input(Some(String::from("a")))
                .interpret()
                .unwrap(),
            String::from("a")
        );
    }

    #[test]
    fn loop_math() {
        assert_eq!(
            Exec::prog("+++++[>++++++++++<-]>-.").interpret().unwrap(),
            String::from("1")
        );
    }

    #[test]
    #[should_panic]
    fn out_of_bounds() {
        Exec::prog("<+").interpret().unwrap();
    }

    #[test]
    #[should_panic]
    fn out_of_input() {
        Exec::prog(",").interpret().unwrap();
    }
}
