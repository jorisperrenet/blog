// Shared navigation state.
export type SiteSection = 'home' | 'about' | 'blog' | 'project' | '';

export type SiteMenuItem = {
	href: string;
	label: string;
	current?: boolean;
};
