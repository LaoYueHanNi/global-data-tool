export interface CityResult {
  geonameid: number;
  name: string;
  name_cn: string | null;
  country_iso: string;
  country_name: string;
  country_name_cn: string | null;
  timezone: string;
  local_time: string;
  local_date: string;
  utc_offset: string;
  is_capital: boolean;
  population: number;
  latitude: number;
  longitude: number;
  feature_code: string | null;
}

export interface CitySummary {
  geonameid: number;
  name: string;
  name_cn: string | null;
  timezone: string;
  local_time: string;
  is_capital: boolean;
  feature_code: string | null;
}

export interface CountryWithCities {
  country_iso: string;
  country_name: string;
  country_name_cn: string | null;
  continent: string | null;
  cities: CitySummary[];
}

export interface SearchResponse {
  cities: CityResult[];
  countries: CountryWithCities[];
}

export interface ExchangeRates {
  base: string;
  rates: Record<string, number>;
  updated_at: string;
  next_update: string;
}
