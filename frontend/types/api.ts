export type Folder = {
    id: string;
    parent_id: string | null;
    owner_id: string;
    name: string;
    created_at: string;
    updated_at: string;
};

export type File = {
    id: string;
    folder_id: string | null;
    owner_id: string;
    owner_name: string;
    name: string;
    size: number;
    created_at: string;
    updated_at: string;
};

export type TypedFolder = {
    type: 'folder';
    item: Folder;
};
export type TypedFile = {
    type: 'file';
    item: File;
};
export type TypedItem = TypedFolder | TypedFile;

export type ApiResponse = {
    success: boolean;
    err_id: string | null;
    detail: string | null;
}

export type UuidName = {
    id: string;
    name: string;
}

export type RefreshToken = {
    id: string;
    user_agent: string | null;
    country: string | null;
    region: string | null;
    city: string | null;
    expires_at: string;
    created_at: string;
}

export type UserAll = {
    id: string;
    login: string;
    username: string;
    admin: boolean;
    created_at: string;
    updated_at: string;

}