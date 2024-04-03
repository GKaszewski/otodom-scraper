export type Offer = {
  id: number;
  title: string;
  price: number;
  area: number;
  rooms: number;
  location: string;
  price_per_m2: number;
  detail_url: string;
};

export type OfferParams = {
  location?: string;
  price?: number;
  rooms?: number;
  exclude?: boolean;
};
