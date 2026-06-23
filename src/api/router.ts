import { CoreEngine } from '../core/engine';

interface RequestPayload {
  id: string;
}

export const router = {
  handle: (req: RequestPayload): void => {
    new CoreEngine().processTx(req.id);
  },
};
