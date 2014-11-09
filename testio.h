typedef struct ffolder{
    char name[32];
    long sector;
    long insector;
    long upsector;
    long next;
    long prev;
    long filesector;
    char properties[3];
    int size;
}ffolder;
typedef struct ffile{
    char name[32];
    long next;
    long prev;
    long sector;
    long fsector;
    char properties[4];
    int size;
}ffile;
typedef struct freesec{
    long sector;
    struct freesec *next;
}freesec;
void delUsedSector(freesec *s);
void putFreeSector(freesec *s,long sector);
void getLastSector(char *path,freesec *s);
long getFreeSector(freesec *s);
freesec *foldsec,*filesec,*fsec;
ffolder foldTravel(char *path,long sector);
ffile fileTravel(char *path,long sector);
ffolder readFolder(char *path,long sector){
    FILE *f;
    ffolder tem;
    f=fopen(path,"r+");
    if(sector!=0)
    {
        if(fseek(f,sector,0)!=0)
            printf("sector error .. this sector does not exists");
        else
            fread(&tem,sizeof(tem),1,f);
    }
    else
    {
        puts("No such folder exists .... system restoring.. ");
        fseek(f,1,0);
        fread(&tem,sizeof(tem),1,f);
    }
    fclose(f);
    return tem;
}
int writeFolder(char *path,ffolder cur,ffolder fold){
    FILE *f;
    long sector;
    ffolder tem;
    f=fopen(path,"r+");
    if(fseek(f,0,2)!=0)
        printf("sector error .. this sector does not exists\n");
    else
    {
        sector=getFreeSector(foldsec);
        delUsedSector(foldsec);
        if(foldsec==0)
            putFreeSector(foldsec,ftell(f));
        fold.upsector=cur.sector;
        fold.sector=sector;
        if(cur.insector==0)
        {
            cur.insector=sector;
            fseek(f,cur.sector,0);
            fwrite(&cur,sizeof(cur),1,f);
        }
        else
        {
            tem=foldTravel(path,cur.insector);
            while(tem.next!=0)
            {
                if(strcmp(tem.name,fold.name)==0)
                {
                    printf("A folder with same name already exists in %s\n",cur.name);
                    fclose(f);
                    return 0;
                }
                
                tem=foldTravel(path,tem.next);
            }
            if(strcmp(tem.name,fold.name)==0)
            {
                printf("A folder with same name already exists in %s\n",cur.name);
                fclose(f);
                return 0;
            }
            tem.next=sector;
            fseek(f,tem.sector,0);
            fold.prev=tem.sector;
            fwrite(&tem,sizeof(tem),1,f);
            fclose(f);
            f=fopen(path,"r+");
        }
        fseek(f,sector,0);
        fwrite(&fold,sizeof(fold),1,f);
        fclose(f);
        getLastSector(path,foldsec);
    }
    fclose(f);
    return 0;
}
int writeFile(char *path,ffolder cur,ffile fold){
    FILE *f;
    long sector;
    ffile tem;
    f=fopen(path,"r+");
    if(fseek(f,0,2)!=0)
        printf("sector error .. this sector does not exists\n");
    else
    {
        sector=getFreeSector(foldsec);
        fold.sector=sector;
        if(cur.filesector==0)
        {
            cur.filesector=sector;
            fseek(f,cur.sector,0);
            fwrite(&cur,sizeof(cur),1,f);
        }
        else
        {
            tem=fileTravel(path,cur.filesector);
            while(tem.next!=0)
            {
                if(strcmp(tem.name,fold.name)==0)
                {
                    printf("A file with same name already exists in %s\n",cur.name);
                    fclose(f);
                    return 0;
                }
                
                tem=fileTravel(path,tem.next);
            }
            if(strcmp(tem.name,fold.name)==0)
            {
                printf("A file with same name already exists in %s\n",cur.name);
                fclose(f);
                return 0;
            }
            tem.next=sector;
            fseek(f,tem.sector,0);
            fold.prev=tem.sector;
            fwrite(&tem,sizeof(tem),1,f);
            fclose(f);
            f=fopen(path,"r+");
        }
        fseek(f,0,2);
        fwrite(&fold,sizeof(fold),1,f);
        fclose(f);
    }
    fclose(f);
    return 0;
}
ffolder foldTravel(char *path,long sector){
    
    FILE *f;
    ffolder tem;
    f=fopen(path,"r+");
    fseek(f,sector,0);
    fread(&tem,sizeof(tem),1,f);
    
    fclose(f);
    return tem;
}
ffile fileTravel(char *path,long sector){
    
    FILE *f;
    ffile tem;
    f=fopen(path,"r+");
    fseek(f,sector,0);
    fread(&tem,sizeof(tem),1,f);
    
    fclose(f);
    return tem;
}
ffolder changeFolder(char *path,char *name,long sector){
    
    FILE *f;
    ffolder tem;
    long sec;
    f=fopen(path,"r+");
    sec=sector;
    tem=foldTravel(path,sec);
    tem=foldTravel(path,tem.insector);
    while(tem.next!=0)
    {
        if(strcmp(tem.name,name)==0)
        {
            return tem;
        }
        tem=foldTravel(path,tem.next);
    }
    if(strcmp(tem.name,name)==0)
    {
        return tem;
    }
    printf("No such folder exists\n");
    fclose(f);
    return foldTravel(path,sector);
}
void showFolder(char *path,ffolder tem){
    
    while(tem.upsector!=0)
    {
        printf("%s",tem.name);
        printf("<-");
        tem=foldTravel(path,tem.upsector);
        
    }
    printf("%s :>",tem.name);
}
int showdir(char *path,ffolder cur){
    ffolder tem;
    ffile temp;
    if(cur.insector!=0)
    {
        tem=foldTravel(path,cur.insector);
        while(tem.next!=0)
        {
            printf("%s <dir>\n",tem.name);
            tem=foldTravel(path,tem.next);
        }
        
        printf("%s <dir>\n",tem.name);
        if(cur.filesector==0)
            return 0;
    }
    if(cur.filesector!=0)
    {
        temp=fileTravel(path,cur.filesector);
        while(temp.next!=0)
        {
            puts(temp.name);
            temp=fileTravel(path,temp.next);
        }
        
        puts(temp.name);
        return 0;
    }
    printf("folder is empty\n");
    return 0;
}
ffolder activateFolder()
{
    ffolder root;
    strcpy(root.name,"new psl folder");
    root.sector=0;
    root.upsector=0;
    root.insector=0;
    root.filesector=0;
    root.next=0;
    root.size=0;
    root.prev=0;
    root.properties[0]='r';
    root.properties[1]='w';
    root.properties[2]='v';
    return root;
}
ffile activateFile(){
    ffile tem;
    strcpy(tem.name,"new file.txt");
    tem.next=0;
    tem.prev=0;
    tem.sector=0;
    tem.fsector=0;
    strcpy(tem.properties,"rwv");
    tem.size=0;
    return tem;
}
char *getCommand()
{
    char *c,a;
    int i=0;
    c=(char*)malloc(sizeof(char)*3);
    c[i]=0;
    while(1)
    {
        a=getch();
        if(a==13||a==10)
        {
            putch('\n');
            break;
        }
        else
        {
            if(c[i]!='\n'&&c[i]!='\r')
                c[i]=a;
            else
                c[i]='\r';
            if(c[i]!='\n'&&c[i]!='\r')
                putch(c[i]);
            else
                printf("\n");
            i++;
            c=(char*)realloc(c,sizeof(c)*(i+2));
            c[i]=0;
        }
    }
    return c;
}
char *content()
{
    char *c,a;
    int i=0;
    c=(char*)malloc(sizeof(char)*3);
    c[i]=0;
    while(1)
    {
        a=getch();
        if(a==26)
            break;
        else
        {
            if(c[i]!='\n'&&c[i]!='\r')
                c[i]=a;
            else
                c[i]='\r';
            if(c[i]!='\n'&&c[i]!='\r')
                putch(c[i]);
            else
                printf("\n");
            i++;
            c=(char*)realloc(c,sizeof(c)*(i+2));
            c[i]=0;
        }
    }
    return c;
}
void invokeFreeSectors(char *path1,char *path2,char *path3){
    FILE *f;
    
    freesec a,*temp=0,*p=0;
    f=fopen(path1,"r+");
    while(fread(&a,sizeof(a),1,f)==1)
    {
        p=temp;
        temp=(freesec*)malloc(sizeof(temp));
        temp->sector=a.sector;
        temp->next=0;
        if(p!=0)
            p->next=temp;
        if(foldsec==0)
            foldsec=temp;
        
    }
    fclose(f);
    f=fopen(path2,"r+");
    p=0;temp=0;
    while(fread(&a,sizeof(a),1,f)==1)
    {
        p=temp;
        temp=(freesec*)malloc(sizeof(temp));
        temp->sector=a.sector;
        temp->next=0;
        if(p!=0)
            p->next=temp;
        if(filesec==0)
            filesec=temp;
    }
    fclose(f);
    f=fopen(path3,"r+");
    p=0;temp=0;
    while(fread(&a,sizeof(a),1,f)==1)
    {
        p=temp;
        temp=(freesec*)malloc(sizeof(temp));
        temp->sector=a.sector;
        temp->next=0;
        if(p!=0)
            p->next=temp;
        if(fsec==0)
            fsec=temp;
    }
    fclose(f);
}
void outvokeFreeSectors(char *path1,char *path2,char *path3){
    FILE *f;
    
    freesec a,*temp,*p;
    f=fopen(path1,"wb");
    temp=foldsec;
    
    while(temp!=0)
    {
        
        fwrite(temp,sizeof(freesec),1,f);
        temp=temp->next;
    }
    fclose(f);
    f=fopen(path2,"wb");
    temp=filesec;
    while(temp!=0)
    {
        fwrite(temp,sizeof(freesec),1,f);
        temp=temp->next;
    }
    fclose(f);
    f=fopen(path3,"wb");
    temp=fsec;
    while(temp!=0)
    {
        fwrite(temp,sizeof(freesec),1,f);
        temp=temp->next;
    }
    fclose(f);
}
long getFreeSector(freesec *s){
    if(s!=0)
        return s->sector;
    
}
void delUsedSector(freesec *s){
    freesec *tem;
    if(foldsec!=0)
    {
        tem = foldsec;
        foldsec=tem->next;
        
        free(tem);
    }
}
void putFreeSector(freesec *s,long sector){
    freesec *tem=0;
    tem=(freesec*)malloc(sizeof(tem));
    tem->next=s;
    tem->sector=sector;
    foldsec=tem;
}
void getLastSector(char *path,freesec *s){
    FILE *f;
    freesec *tem;
    f=fopen(path,"r+");
    fseek(f,0,2);
    tem=s;
    while(tem->next!=0)
        tem=tem->next;
    tem->sector=ftell(f);
    fclose(f);
    outvokeFreeSectors("test1.fol","test1.fil","test1.fs");
}
int deletefold(char *path,ffolder cur,char *name){
    FILE *f;
    long sector;
    ffolder tem,fold;
    f=fopen(path,"r+");
    
    fold.upsector=cur.sector;
    fold.sector=sector;
    tem=foldTravel(path,cur.insector);
    while(tem.next!=0)
    {
        if(strcmp(tem.name,name)==0)
        {
            if(tem.sector==cur.insector)
            {
                
                cur.insector=tem.next;
                fseek(f,cur.sector,0);
                fwrite(&cur,sizeof(cur),1,f);
                fold=tem;
            }
            else
            {
                
                fold=tem;
                tem=foldTravel(path,fold.prev);
                tem.next=fold.next;
                fseek(f,tem.sector,0);
                fwrite(&tem,sizeof(tem),1,f);
                tem=foldTravel(path,fold.next);
                tem.prev=fold.prev;
                fseek(f,tem.sector,0);
                fwrite(&tem,sizeof(tem),1,f);
                
            }
            
            putFreeSector(foldsec,fold.sector);
            
            outvokeFreeSectors("test1.fol","test1.fil","test1.fs");
            
        }
        
        tem=foldTravel(path,tem.next);
    }
    if(strcmp(tem.name,name)==0)
    {
        
        if(tem.sector==cur.insector)
        {
            
            if(tem.next!=0)
                cur.insector=tem.next;
            else
                cur.insector=0;
            fseek(f,cur.sector,0);
            //printf("cur.in=%d",cur.insector);
            fwrite(&cur,sizeof(cur),1,f);
            fold=tem;
        }
        else
        {
            
            fold=tem;
            tem=foldTravel(path,fold.prev);
            tem.next=fold.next;
            fseek(f,tem.sector,0);
            fwrite(&tem,sizeof(tem),1,f);
            /*
             tem=foldTravel(path,fold.next);
             tem.prev=fold.prev;
             fseek(f,tem.sector,0);
             fwrite(&tem,sizeof(tem),1,f);
             */
        }
        putFreeSector(foldsec,fold.sector);
        outvokeFreeSectors("test1.fol","test1.fil","test1.fs");
        fclose(f);
        return 0;
    }
    
    
    puts("Cannot delete because..no such folder exists..");
    
    
    
    fclose(f);
    return 0;
}

