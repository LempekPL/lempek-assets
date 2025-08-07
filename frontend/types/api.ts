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
    name: string;
    size: number;
    created_at: string;
    updated_at: string;
};

export type ApiResponse = {
    success: boolean;
    err_id: string | null;
    detail: string | null;
}

export type UuidName = {
    id: string;
    name: string;
}