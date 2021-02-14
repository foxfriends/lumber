use super::*;

pub(crate) trait Variables {
    fn variables(&self, vars: &mut Vec<Variable>);

    fn get_variables(&self) -> Vec<Variable> {
        let mut vec = vec![];
        self.variables(&mut vec);
        vec
    }
}
