import useAppStore from '@/lib/store';
import { Input } from './ui/input';
import { Label } from './ui/label';
import { ToggleGroup, ToggleGroupItem } from './ui/toggle-group';
import { Checkbox } from './ui/checkbox';

const TopBar = () => {
  const location = useAppStore((state) => state.location);
  const setLocation = useAppStore((state) => state.setLocation);
  const price = useAppStore((state) => state.price);
  const setPrice = useAppStore((state) => state.setPrice);
  const rooms = useAppStore((state) => state.rooms);
  const setRooms = useAppStore((state) => state.setRooms);
  const exclude = useAppStore((state) => state.exclude);
  const setExclude = useAppStore((state) => state.setExclude);

  return (
    <div className="p-4 flex gap-4 items-center w-full">
      <div className="flex flex-col gap-1">
        <Label htmlFor="location">Lokalizacja</Label>
        <div className="inline-flex gap-2 items-center">
          <Input
            id="location"
            onChange={(event) => setLocation(event.target.value)}
            value={location || ''}
          />
          <Label htmlFor="exclude">Wyklucz</Label>
          <Checkbox
            id="exclude"
            checked={exclude}
            onCheckedChange={() => setExclude(!exclude)}
          />
        </div>
      </div>
      <div className="flex flex-col gap-1 w-40">
        <Label htmlFor="price">Cena</Label>
        <Input
          id="price"
          onChange={(event) => setPrice(Number(event.target.value))}
          value={price || ''}
        />
      </div>
      <div className="flex flex-col gap-1">
        <Label htmlFor="rooms">Pokoje</Label>
        <ToggleGroup
          variant="outline"
          type="single"
          className="w-full"
          value={rooms?.toString() || String(2)}
          onValueChange={(val) => setRooms(Number(val))}
        >
          <ToggleGroupItem value="1">1</ToggleGroupItem>
          <ToggleGroupItem value="2">2</ToggleGroupItem>
          <ToggleGroupItem value="3">3</ToggleGroupItem>
          <ToggleGroupItem value="4">4</ToggleGroupItem>
          <ToggleGroupItem value="5">5</ToggleGroupItem>
        </ToggleGroup>
      </div>
    </div>
  );
};

export default TopBar;
