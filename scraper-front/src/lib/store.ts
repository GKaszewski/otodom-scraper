import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface State {
  location: string | undefined;
  price: number | undefined;
  rooms: number | undefined;
  exclude: boolean | undefined;

  setLocation: (location: string | undefined) => void;
  setPrice: (price: number | undefined) => void;
  setRooms: (rooms: number | undefined) => void;
  setExclude: (exclude: boolean | undefined) => void;
}

const useAppStore = create<State>()(
  persist(
    (set) => ({
      location: '',
      price: 0,
      rooms: 0,
      exclude: false,

      setLocation: (location) => set({ location }),
      setPrice: (price) => set({ price }),
      setRooms: (rooms) => set({ rooms }),
      setExclude: (exclude) => set({ exclude }),
    }),
    {
      name: 'scraper-front',
    }
  )
);

export default useAppStore;
