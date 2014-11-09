typedef struct file{
    char name[32];
    struct file *next;
    int sector;
    char properties[4];
    int size;
}file;

typedef struct ffolder{
    char name[32];
    int sector;
    int insector;
    int upsector;
    int next;
    int filesector;
    char properties[3];
    int size;
}ffolder;
typedef struct folder{
    char name[32];
    int sector;
    struct folder *in;
    struct folder *up;
    struct folder *next;
    struct file *fin;
    char properties[3];
    int size;
}folder;

typedef struct ffile{
    char name[32];
    int next;
    int sector;
    char properties[4];
    int size;
}ffile;
folder *first=0,*temp=0,*ptr=0,*curptr=0;
file *fptr,*ftemp;
ffile fout;
long cursector=1;
int readfs(char *path){
    ffolder fold;
    FILE *f;
    ffolder temp1;
    
    
    f=fopen(path,"r+");
    ptr=first;
    while(1)
    {
        
        if(first==0)
        {
            fseek(f,cursector,0);
            fread(&temp1,sizeof(temp1),1,f);
            cursector=cursector+sizeof(temp1);
            first=(folder*)malloc(sizeof(folder));
            strcpy(first->name,temp1.name);
            first->sector=temp1.sector;
            first->up=0;
            first->next=0;
            strcpy(first->properties,temp1.properties);
            first->in=0;
            first->fin=0;
            first->size=0;
            ptr=first;
            fold=temp1;
        }
        else
        {
            
            ptr=temp;
            temp1=fold;
            fseek(f,cursector,0);
            if(fread(&fold,sizeof(fold),1,f)!=1)
                return 0;
            cursector=cursector+sizeof(fold);
            temp=(folder*)malloc(sizeof(folder));
            strcpy(temp->name,fold.name);
            temp->sector=fold.sector;
            temp->up=ptr;
            temp->next=0;
            strcpy(temp->properties,fold.properties);
            temp->in=0;
            temp->fin=0;
            temp->size=0;
            if(temp1.insector!=0)
            {
                if(ptr==0)
                    ptr=first;
                ptr->in=temp;
                
            }
            else if(temp1.next!=0)
            {
                ptr->next=temp;
                
            }
            else if(temp1.filesector!=0)
            {
                while(fout.sector!=0)
                    
                {
                    fptr=ftemp;
                    ftemp=(file*)malloc(sizeof(file));
                    fread(&fout,sizeof(fout),1,f);
                    strcpy(ftemp->name,fout.name);
                    ftemp->size=fout.size;
                    ftemp->sector=fout.sector;
                    strcpy(ftemp->properties,fout.properties);
                    ftemp->size=fout.size;
                    ftemp->next=0;
                    if(ptr->fin==0)
                    {
                        ptr->fin=ftemp;
                    }
                    else if(fptr!=0)
                    {
                        fptr->next=ftemp;
                    }
                }
            }
            else if(ptr->up!=0)
            {
                ptr=ptr->up;
            }
            
            ftemp=0;
            cursector=ftell(f);
        }
    }
}
void showCurrentfolder(folder *cur){
    while(cur->up!=0)
    {
        printf("%s",cur->name);
        cur=cur->up;
        printf("<-");
    }
    printf("%s",cur->name);
    printf(":>");
}
int makedir(char *name){
    temp=(folder*)malloc(sizeof(folder));
    strcpy(temp->name,name);
    temp->sector=0;
    temp->up=curptr;
    temp->next=0;
    strcpy(temp->properties,"rwv");
    temp->in=0;
    temp->fin=0;
    temp->size=0;
    ptr=curptr->in;
    if(ptr!=0)
    {
        if(strcmp(name,ptr->name)==0)
        {
            printf("a folder with same name already exists\n");
            free(temp);
            return 0;//a folder with same name exists
        }
        while(ptr->next!=0)
        {
            
            
            ptr=ptr->next;
            if(strcmp(name,ptr->name)==0)
            {
                printf("a folder with same name already exists\n");
                free(temp);
                return 0;//a folder with same name exists
            }
        }
        ptr->next=temp;
    }
    else
    {
        
        curptr->in=temp;
    }
}
int changedir(char *name)
{
    //puts(name);
    ptr=curptr->in;
    if(ptr!=0)
    {
        while(ptr!=0)
        {
            
            if(strcmp(name,ptr->name)==0)
            {
                
                curptr=ptr;//changed current folder
                break;
            }
            ptr=ptr->next;
        }
    }
}
void showdir(){
    int d=0,f=0;
    file *fi;
    printf("\n");
    ptr=curptr->in;
    
    if(ptr!=0)
        while(ptr!=0)
        {
            printf("%s <dir>\n",ptr->name);
            ptr=ptr->next;
            d=d+1;
        }
    
    fi=curptr->fin;
    
    if(fi!=0)
        while(fi!=0)
        {
            printf("%s <file>\n",fi->name);
            fi=(file*)fi->next;
            f=f+1;
        }
    printf("\n");
    printf("Total number of directories = %d \nand files = %d\n\n",d,f);
}
int writefs(char *path){
    FILE *f;
    
    long fbrk=0,fsector;
    ffolder in;
    ptr=first;
    fptr=0;
    f=fopen(path,"r+");
    fseek(f,1,0);
    cursector=fbrk=1;
    while(1)
    {
        puts(ptr->name);
        strcpy(in.name,ptr->name);
        in.sector=cursector;
        in.upsector=0;
        in.next=0;
        strcpy(in.properties,ptr->properties);
        in.insector=0;
        in.filesector=0;
        in.size=ptr->size;
        if(ptr->up!=0)
        {
            in.upsector=fbrk;
        }
        if(ptr->in!=0)
        {
            
            cursector=cursector+sizeof(ffolder);
            in.insector=cursector;
            temp=ptr->in;
            while(temp!=0)
            {
                cursector=cursector+sizeof(ffolder);
                temp=temp->next;
            }
            
        }
        if(ptr->fin!=0)
        {
            in.filesector=cursector;
            ftemp=ptr->fin;
            fsector=cursector;
            while(ftemp!=0)
            {
                cursector=cursector+sizeof(ffile);
            }
        }               
        if(ptr->next!=0)
        {
            in.next=cursector;
        }
        if(ptr->in!=0)
            ptr=ptr->in; 
        else if(ptr->fin!=0)
            fptr=ptr->fin;
        else if(ptr->next!=0)
            ptr=ptr->next;
        else if(ptr->up==0)
        {
            fclose(f);
            return 0;
        }
        else if(ptr->up->next!=0)
            ptr=ptr->up->next;
        else
        {   fclose(f);
            fptr=0;
            return 0;
        }
        fwrite(&in,sizeof(in),1,f);
        if(fptr!=0)
        {
            while(fptr!=0)
            {
                strcpy(fout.name,fptr->name);
                fout.next=fsector;
                strcpy(fout.properties,fptr->properties);
                fout.size=fptr->size;
                fsector=fsector+sizeof(ffile);
                if(fptr->next!=0)
                    fout.next=fsector;
                fptr=fptr->next;
                fwrite(&fout,sizeof(fout),1,f);
            }
        }
    }    
    fclose(f);
}
