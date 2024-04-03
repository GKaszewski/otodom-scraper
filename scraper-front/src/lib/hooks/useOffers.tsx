import { useQuery } from '@tanstack/react-query';
import { fetchOffers } from '../api/fetch_offers';
import { OfferParams } from '../api/types';

export const useOffers = (params: OfferParams | null) => {
  const QUERY_KEY = `offers/${JSON.stringify(params) || ''}`;

  return useQuery({
    queryKey: [QUERY_KEY],
    queryFn: () => fetchOffers(params),
    initialData: [],
  });
};

export default useOffers;
