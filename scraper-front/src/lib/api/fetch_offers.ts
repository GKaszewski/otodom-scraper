import { Offer, OfferParams } from './types';

export const fetchOffers = async (params: OfferParams | null) => {
  const endpoint = import.meta.env.VITE_API_ENDPOINT;
  try {
    let response;
    const urlParams = new URLSearchParams();
    if (params?.location) {
      urlParams.set('location', params.location);
      if (params?.price) {
        urlParams.set('price', params.price.toString());
      }
      if (params?.exclude) {
        urlParams.set('exclude', params.exclude.toString());
      }
      if (params?.rooms) {
        urlParams.set('rooms', params.rooms.toString());
      }
    }

    if (params) {
      response = await fetch(`${endpoint}/offers?${urlParams}`);
    } else {
      response = await fetch(`${endpoint}/offers`);
    }
    if (!response.ok) {
      throw new Error(response.statusText);
    }

    return (await response.json()) as Offer[];
  } catch (error) {
    console.error(error);
    return [];
  }
};
