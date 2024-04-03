import useOffers from '@/lib/hooks/useOffers';
import { Skeleton } from './ui/skeleton';
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import useAppStore from '@/lib/store';

const Offers = () => {
  const { price, location, exclude, rooms } = useAppStore();
  const { data, isLoading, isError, error } = useOffers({
    price,
    location,
    exclude,
    rooms,
  });

  if (isLoading) {
    return (
      <div className="flex flex-col gap-2">
        <Skeleton />
        <Skeleton />
        <Skeleton />
      </div>
    );
  }

  if (isError) {
    return <div>Error: {error.message}</div>;
  }

  const prettyPrice = (price: number) => {
    return price.toLocaleString('pl-PL');
  };

  if (!data) {
    return <div>Brak ofert</div>;
  }

  return (
    <Table className="w-full h-full mx-4">
      <TableCaption>Oferty mieszkań</TableCaption>
      <TableHeader>
        <TableRow>
          <TableHead>Lokalizacja</TableHead>
          <TableHead>Cena</TableHead>
          <TableHead>Cena za m2</TableHead>
          <TableHead>Metraż</TableHead>
          <TableHead>Pokoje</TableHead>
          <TableHead>Link</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {data?.map((offer) => (
          <TableRow key={offer.id}>
            <TableCell>{offer.location}</TableCell>
            <TableCell>{prettyPrice(offer.price)} zł</TableCell>
            <TableCell>{prettyPrice(offer.price_per_m2)} zł</TableCell>
            <TableCell>{offer.area} m2</TableCell>
            <TableCell>{offer.rooms}</TableCell>
            <TableCell>
              <a href={offer.detail_url} target="_blank" rel="noreferrer">
                Link
              </a>
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
};

export default Offers;
