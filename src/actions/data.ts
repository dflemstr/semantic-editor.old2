import actionCreatorFactory from 'typescript-fsa';

const actionCreator = actionCreatorFactory('semantic-editor/data');

export const fetchSlateSchema = actionCreator.async<{}, {}, {}>('FETCH_SLATE_SCHEMA');
export const fetchSnapshot = actionCreator.async<{}, {}, {}>('FETCH_SNAPSHOT');
