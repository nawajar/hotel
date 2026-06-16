-- Admin-editable overrides for frontend UI labels. Default text lives in the
-- frontend's bundled locale catalogs; a row here overrides one (key, locale)
-- pair. Absence of a row means "use the frontend default".
create table translation_overrides
(
    key        text not null,
    locale     text not null check (locale in ('en', 'lo')),
    value      text not null,
    updated_at timestamptz not null default now(),
    primary key (key, locale)
);
