use crate::{app::State, data::Data, Args};

#[derive(Default)]
pub struct Solution {
    pub solutions: Vec<(String, f32)>
}
impl From<State> for Solution {
    fn from(state: State) -> Self {
        let args: Args = state.args;
        let data: Data = state.data;
        let anagram: String = args.anagram;

        if args.word_count != Some(0) {
            
        } else {

        }





        let mut solution: Solution = Solution::default();
        solution
    }
}